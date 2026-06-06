#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main(flavor = "current_thread")]
async fn main() -> std::process::ExitCode {
    match native::run(std::env::args().skip(1).collect()).await {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            std::process::ExitCode::FAILURE
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use std::{
        env, fs,
        net::SocketAddr,
        path::{Path, PathBuf},
        time::Duration,
    };

    use axum::{Router, routing::get};
    use az_aio_plugin_sync::{
        FileSystemSyncObjectStore, RustfsSyncObjectStore, SyncAgentConfig, SyncAgentRoot,
        SyncAgentRootsConfig, SyncDeviceInfo, SyncEngine, SyncError,
        SyncFileSystemObjectStoreConfig, SyncObjectManifest, SyncObjectStoreConfig,
        SyncPgRepository, SyncRootWatcher, SyncWatchPlanner, SyncWireMessage, SyncWsConnection,
        SyncWsReader, SyncWsWriter, bootstrap_sync_agent, build_sync_agent_engine, sync_api_router,
        sync_model::normalize_home_relative_path,
    };
    use az_rustfs::S3ClientConfig;
    use serde::Serialize;
    use thiserror::Error;
    use tokio::net::TcpListener;

    const DEFAULT_BIND_ADDR: &str = "127.0.0.1:8787";

    pub async fn run(args: Vec<String>) -> Result<(), SyncCliError> {
        match SyncCliCommand::parse(args)? {
            SyncCliCommand::Serve(args) => serve(args).await,
            SyncCliCommand::Agent(args) => agent(args).await,
            SyncCliCommand::RootAdd(args) => root_add(args),
        }
    }

    async fn serve(args: ServeArgs) -> Result<(), SyncCliError> {
        let config = args.agent_config()?;
        let engine = build_sync_agent_engine(&config)?;
        if config.write_local_index {
            engine.write_default_local_index()?;
        }
        if config.write_finder_state {
            engine.write_default_finder_state()?;
        }
        let bind = args.bind.parse::<SocketAddr>()?;
        let mut state = az_aio_plugin_sync::SyncApiState::new(engine)
            .with_optional_auth_token(args.auth_token());
        if let Some(database_url) = args.database_url() {
            let pool = sqlx::PgPool::connect(&database_url).await?;
            let repository = SyncPgRepository::new(pool);
            repository.migrate().await?;
            state = state.with_pg_repository(repository);
        }
        if let Some(object_config) = args.object_store_config() {
            let store = object_config.build_store();
            store.ensure()?;
        }
        let app = Router::new()
            .route("/health", get(|| async { "ok" }))
            .merge(sync_api_router(state));
        let listener = TcpListener::bind(bind)
            .await
            .map_err(|source| SyncCliError::io_error("bind sync server", None, source))?;
        let local_addr = listener.local_addr().map_err(|source| {
            SyncCliError::io_error("read sync server local address", None, source)
        })?;
        println!("az-aio sync serve");
        println!("device: {}", config.device.device_name);
        println!("home: {}", config.device.home_dir.display());
        println!("listening: http://{local_addr}");
        println!("status: http://{local_addr}/api/sync/status");
        axum::serve(listener, app)
            .await
            .map_err(|source| SyncCliError::io_error("serve sync api", None, source))?;
        Ok(())
    }

    async fn agent(args: AgentArgs) -> Result<(), SyncCliError> {
        let config = args.agent_config()?;
        let mut engine = build_sync_agent_engine(&config)?;
        persist_agent_state(&engine, config.write_local_index, config.write_finder_state)?;
        print_json(&AgentReport::from_engine("bootstrapped", &engine)?)?;
        if args.once {
            return Ok(());
        }

        let mut planner = SyncWatchPlanner::new(config.device.clone());
        for file in engine.files() {
            planner.remember_record(&file);
        }

        let mut remote_writer = None;
        let mut remote_reader = None;
        let object_store = args
            .object_store_config()
            .map(|config| {
                let store = config.build_store();
                store.ensure()?;
                Ok::<_, SyncCliError>(store)
            })
            .transpose()?;
        if let Some(endpoint) = args.endpoint() {
            let connection =
                SyncWsConnection::connect(&endpoint, args.auth_token().as_deref()).await?;
            let SyncWsConnection { mut writer, reader } = connection;
            writer
                .send(&SyncWireMessage::Hello {
                    device: config.device.clone(),
                    roots: engine.roots(),
                })
                .await?;
            send_existing_text_updates(&engine, &mut writer).await?;
            send_existing_binary_manifests(
                &engine,
                &mut planner,
                object_store.as_ref(),
                &mut writer,
            )
            .await?;
            remote_writer = Some(writer);
            remote_reader = Some(reader);
        }

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let roots = engine.roots();
        let _watcher = SyncRootWatcher::watch_roots(roots, move |event| {
            let _ = tx.send(event);
        })?;

        println!("az-aio sync agent");
        println!("device: {}", config.device.device_name);
        println!("home: {}", config.device.home_dir.display());
        println!("watching roots: {}", engine.roots().len());

        loop {
            tokio::select! {
                event = rx.recv() => {
                    let Some(event) = event else {
                        return Err(SyncCliError::WatchChannel("sync watcher channel closed".to_string()));
                    };
                    planner.push(event);
                    tokio::time::sleep(planner.debounce_window().max(Duration::from_millis(1))).await;
                    while let Ok(event) = rx.try_recv() {
                        planner.push(event);
                    }
                    let plan = planner.drain_plan()?;
                    if plan.is_empty() {
                        continue;
                    }
                    let changed_text_paths = planner.apply_plan(&mut engine, &plan)?;
                    send_local_plan_updates(
                        &mut engine,
                        &mut planner,
                        &changed_text_paths,
                        &plan.changed_binary_paths,
                        &plan.deleted_paths,
                        object_store.as_ref(),
                        remote_writer.as_mut(),
                    ).await?;
                    persist_agent_state(&engine, config.write_local_index, config.write_finder_state)?;
                    print_json(&AgentReport::from_engine("changed", &engine)?)?;
                }
                message = recv_remote_message(&mut remote_reader), if remote_reader.is_some() => {
                    match message? {
                        Some(message) => {
                            handle_remote_message(
                                &mut engine,
                                &mut planner,
                                remote_writer.as_mut(),
                                message,
                                object_store.as_ref(),
                                config.write_local_index,
                                config.write_finder_state,
                            ).await?;
                        }
                        None => {
                            remote_reader = None;
                            remote_writer = None;
                        }
                    }
                }
            }
        }
    }

    async fn recv_remote_message(
        remote_reader: &mut Option<SyncWsReader>,
    ) -> Result<Option<SyncWireMessage>, SyncCliError> {
        let Some(reader) = remote_reader.as_mut() else {
            return Ok(None);
        };
        reader.recv().await.map_err(Into::into)
    }

    async fn send_local_plan_updates(
        engine: &mut SyncEngine,
        planner: &mut SyncWatchPlanner,
        changed_text_paths: &[String],
        changed_binary_paths: &[String],
        deleted_paths: &[String],
        object_store: Option<&SyncCliObjectStore>,
        remote_writer: Option<&mut SyncWsWriter>,
    ) -> Result<(), SyncCliError> {
        for relative_path in deleted_paths {
            let _ = engine.delete_file(relative_path)?;
        }
        let Some(writer) = remote_writer else {
            return Ok(());
        };
        for relative_path in changed_text_paths {
            if let Ok(envelope) = engine.export_update_since(relative_path, None) {
                writer.send(&SyncWireMessage::Update { envelope }).await?;
            }
        }
        for relative_path in deleted_paths {
            writer
                .send(&SyncWireMessage::FileDeleted {
                    relative_path: relative_path.clone(),
                    source_device: engine.device().device_name.clone(),
                })
                .await?;
        }
        if changed_binary_paths.is_empty() {
            return Ok(());
        }
        let Some(object_store) = object_store else {
            return Err(SyncCliError::ObjectStoreUnavailable);
        };
        for relative_path in changed_binary_paths {
            let local_path = engine
                .device()
                .local_path_for_home_relative(relative_path)?;
            let manifest = object_store.put_file(
                space_id_for_relative_path(engine, relative_path),
                relative_path,
                &local_path,
            )?;
            if planner.known_content_hash(relative_path) == Some(manifest.content_hash.as_str()) {
                continue;
            }
            writer
                .send(&SyncWireMessage::ObjectManifest {
                    manifest: manifest.clone(),
                    source_device: engine.device().device_name.clone(),
                })
                .await?;
            planner.remember_object_manifest(&manifest);
        }
        Ok(())
    }

    async fn send_existing_text_updates(
        engine: &SyncEngine,
        writer: &mut SyncWsWriter,
    ) -> Result<(), SyncCliError> {
        for file in engine.files() {
            let envelope = engine.export_update_since(&file.relative_path, None)?;
            writer.send(&SyncWireMessage::Update { envelope }).await?;
        }
        Ok(())
    }

    async fn send_existing_binary_manifests(
        engine: &SyncEngine,
        planner: &mut SyncWatchPlanner,
        object_store: Option<&SyncCliObjectStore>,
        writer: &mut SyncWsWriter,
    ) -> Result<(), SyncCliError> {
        let binary_paths = collect_existing_binary_paths(engine)?;
        if binary_paths.is_empty() {
            return Ok(());
        }
        let Some(object_store) = object_store else {
            return Err(SyncCliError::ObjectStoreUnavailable);
        };
        for relative_path in binary_paths {
            let local_path = engine
                .device()
                .local_path_for_home_relative(&relative_path)?;
            let manifest = object_store.put_file(
                space_id_for_relative_path(engine, &relative_path),
                &relative_path,
                &local_path,
            )?;
            if planner.known_content_hash(&relative_path) == Some(manifest.content_hash.as_str()) {
                continue;
            }
            writer
                .send(&SyncWireMessage::ObjectManifest {
                    manifest: manifest.clone(),
                    source_device: engine.device().device_name.clone(),
                })
                .await?;
            planner.remember_object_manifest(&manifest);
        }
        Ok(())
    }

    fn collect_existing_binary_paths(engine: &SyncEngine) -> Result<Vec<String>, SyncCliError> {
        let mut binary_paths = Vec::new();
        for root in engine.roots() {
            let mut pending = vec![root.local_path.clone()];
            while let Some(dir) = pending.pop() {
                let entries = fs::read_dir(&dir).map_err(|source| {
                    SyncCliError::io_error("read sync root directory", Some(dir.clone()), source)
                })?;
                for entry in entries {
                    let entry = entry.map_err(|source| {
                        SyncCliError::io_error("read sync root entry", Some(dir.clone()), source)
                    })?;
                    let path = entry.path();
                    let metadata = entry.metadata().map_err(|source| {
                        SyncCliError::io_error(
                            "read sync root entry metadata",
                            Some(path.clone()),
                            source,
                        )
                    })?;
                    if metadata.is_dir() {
                        pending.push(path);
                    } else if metadata.is_file() && !is_utf8_file(&path)? {
                        let relative_path = engine.device().home_relative_path(&path)?;
                        binary_paths.push(normalize_home_relative_path(&relative_path)?);
                    }
                }
            }
        }
        binary_paths.sort();
        binary_paths.dedup();
        Ok(binary_paths)
    }

    fn is_utf8_file(path: &Path) -> Result<bool, SyncCliError> {
        let bytes = fs::read(path).map_err(|source| {
            SyncCliError::io_error(
                "read sync file for binary classification",
                Some(path.to_path_buf()),
                source,
            )
        })?;
        Ok(String::from_utf8(bytes).is_ok())
    }

    async fn handle_remote_message(
        engine: &mut SyncEngine,
        planner: &mut SyncWatchPlanner,
        remote_writer: Option<&mut SyncWsWriter>,
        message: SyncWireMessage,
        object_store: Option<&SyncCliObjectStore>,
        write_local_index: bool,
        write_finder_state: bool,
    ) -> Result<(), SyncCliError> {
        match message {
            SyncWireMessage::Update { envelope } | SyncWireMessage::Snapshot { envelope } => {
                let record = engine.import_remote_blob(envelope)?;
                engine.materialize_text_to_local_file(&record.relative_path)?;
                planner.remember_record(&record);
                persist_agent_state(engine, write_local_index, write_finder_state)?;
                print_json(&AgentReport::from_engine("remote-update", engine)?)?;
            }
            SyncWireMessage::RequestSnapshot { relative_path } => {
                if let Some(writer) = remote_writer {
                    let envelope = engine.export_snapshot(&relative_path)?;
                    writer.send(&SyncWireMessage::Snapshot { envelope }).await?;
                }
            }
            SyncWireMessage::Error { message } => eprintln!("remote sync error: {message}"),
            SyncWireMessage::ObjectManifest {
                manifest,
                source_device,
            } => {
                if source_device == engine.device().device_name {
                    return Ok(());
                }
                let Some(object_store) = object_store else {
                    return Err(SyncCliError::ObjectStoreUnavailable);
                };
                let local_path = engine
                    .device()
                    .local_path_for_home_relative(&manifest.relative_path)?;
                object_store.materialize_file(&manifest, &local_path)?;
                planner.remember_object_manifest(&manifest);
                if let Some(writer) = remote_writer {
                    writer
                        .send(&SyncWireMessage::Ack {
                            relative_path: manifest.relative_path.clone(),
                            version: manifest.content_hash.clone().into_bytes(),
                        })
                        .await?;
                }
                persist_agent_state(engine, write_local_index, write_finder_state)?;
                print_json(&AgentReport::from_engine("remote-object", engine)?)?;
            }
            SyncWireMessage::FileDeleted {
                relative_path,
                source_device,
            } => {
                if source_device == engine.device().device_name {
                    return Ok(());
                }
                let relative_path = normalize_home_relative_path(&relative_path)?;
                let local_path = engine
                    .device()
                    .local_path_for_home_relative(&relative_path)?;
                remove_local_file_if_exists(&local_path)?;
                let _ = engine.delete_file(&relative_path)?;
                planner.forget_path(&relative_path);
                if let Some(writer) = remote_writer {
                    writer
                        .send(&SyncWireMessage::Ack {
                            relative_path,
                            version: source_device.into_bytes(),
                        })
                        .await?;
                }
                persist_agent_state(engine, write_local_index, write_finder_state)?;
                print_json(&AgentReport::from_engine("remote-delete", engine)?)?;
            }
            SyncWireMessage::Hello { .. }
            | SyncWireMessage::Heartbeat { .. }
            | SyncWireMessage::Ack { .. } => {}
        }
        Ok(())
    }

    fn remove_local_file_if_exists(path: &Path) -> Result<(), SyncCliError> {
        match fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(SyncCliError::io_error(
                "remove synced file",
                Some(path.to_path_buf()),
                error,
            )),
        }
    }

    fn space_id_for_relative_path(engine: &SyncEngine, relative_path: &str) -> String {
        engine
            .roots()
            .into_iter()
            .filter(|root| relative_path_is_inside_root(relative_path, &root.relative_path))
            .max_by_key(|root| root.relative_path.len())
            .map(|root| root.space_id)
            .unwrap_or_else(|| "main".to_string())
    }

    fn relative_path_is_inside_root(relative_path: &str, root_relative_path: &str) -> bool {
        relative_path == root_relative_path
            || relative_path
                .strip_prefix(root_relative_path)
                .is_some_and(|rest| rest.starts_with('/'))
    }

    fn root_add(args: RootAddArgs) -> Result<(), SyncCliError> {
        let device = device_from_options(args.device_name, args.home)?;
        let relative_path = home_relative_input(&device, &args.path)?;
        let alias = args
            .alias
            .unwrap_or_else(|| default_alias_for_relative_path(&relative_path));
        let root = SyncAgentRoot::new(
            alias,
            &relative_path,
            args.space_id.unwrap_or_else(|| "main".to_string()),
        )?;
        let mut roots_config = SyncAgentRootsConfig::read_from_default_path(&device.home_dir)?;
        roots_config.upsert_root(root.clone());
        roots_config.write_to_default_path(&device.home_dir)?;
        let config = SyncAgentConfig::for_device(device).with_root(root);
        let report = bootstrap_sync_agent(config)?;
        print_json(&report)?;
        Ok(())
    }

    fn persist_agent_state(
        engine: &SyncEngine,
        write_local_index: bool,
        write_finder_state: bool,
    ) -> Result<(), SyncCliError> {
        if write_local_index {
            engine.write_default_local_index()?;
        }
        if write_finder_state {
            engine.write_default_finder_state()?;
        }
        Ok(())
    }

    trait SyncApiStateAuthExt {
        fn with_optional_auth_token(self, token: Option<String>) -> Self;
    }

    impl SyncApiStateAuthExt for az_aio_plugin_sync::SyncApiState {
        fn with_optional_auth_token(self, token: Option<String>) -> Self {
            match token {
                Some(token) => self.with_auth_token(token),
                None => self,
            }
        }
    }

    #[derive(Debug, Eq, PartialEq)]
    enum SyncCliCommand {
        Serve(ServeArgs),
        Agent(AgentArgs),
        RootAdd(RootAddArgs),
    }

    impl SyncCliCommand {
        fn parse(mut args: Vec<String>) -> Result<Self, SyncCliError> {
            if matches!(args.first().map(String::as_str), Some("--help" | "-h")) {
                return Err(SyncCliError::Usage(usage()));
            }
            if args.first().map(String::as_str) == Some("sync") {
                args.remove(0);
            }
            let Some(command) = args.first().cloned() else {
                return Err(SyncCliError::Usage(usage()));
            };
            args.remove(0);
            match command.as_str() {
                "serve" => Ok(Self::Serve(ServeArgs::parse(args)?)),
                "agent" => Ok(Self::Agent(AgentArgs::parse(args)?)),
                "root" => parse_root_command(args),
                _ => Err(SyncCliError::Usage(format!(
                    "unknown sync command `{command}`\n\n{}",
                    usage()
                ))),
            }
        }
    }

    fn parse_root_command(mut args: Vec<String>) -> Result<SyncCliCommand, SyncCliError> {
        let Some(command) = args.first().cloned() else {
            return Err(SyncCliError::Usage(usage()));
        };
        args.remove(0);
        match command.as_str() {
            "add" => Ok(SyncCliCommand::RootAdd(RootAddArgs::parse(args)?)),
            _ => Err(SyncCliError::Usage(format!(
                "unknown sync root command `{command}`\n\n{}",
                usage()
            ))),
        }
    }

    #[derive(Debug, Eq, PartialEq)]
    struct ServeArgs {
        bind: String,
        token: Option<String>,
        database_url: Option<String>,
        object_dir: Option<PathBuf>,
        object_endpoint: Option<String>,
        object_bucket: Option<String>,
        object_access_key: Option<String>,
        object_secret_key: Option<String>,
        object_region: Option<String>,
        common: CommonAgentArgs,
    }

    impl ServeArgs {
        fn parse(args: Vec<String>) -> Result<Self, SyncCliError> {
            let mut parser = ArgParser::new(args);
            let bind = parser
                .take_option("--bind")?
                .unwrap_or_else(|| DEFAULT_BIND_ADDR.to_string());
            let token = parser.take_option("--token")?;
            let database_url = parser.take_option("--database-url")?;
            let object_dir = parser.take_option("--object-dir")?.map(PathBuf::from);
            let object_endpoint = parser.take_option("--object-endpoint")?;
            let object_bucket = parser.take_option("--object-bucket")?;
            let object_access_key = parser.take_option("--object-access-key")?;
            let object_secret_key = parser.take_option("--object-secret-key")?;
            let object_region = parser.take_option("--object-region")?;
            let common = CommonAgentArgs::parse(&mut parser)?;
            parser.finish()?;
            Ok(Self {
                bind,
                token,
                database_url,
                object_dir,
                object_endpoint,
                object_bucket,
                object_access_key,
                object_secret_key,
                object_region,
                common,
            })
        }

        fn agent_config(&self) -> Result<SyncAgentConfig, SyncCliError> {
            self.common.agent_config()
        }

        fn auth_token(&self) -> Option<String> {
            self.token
                .clone()
                .or_else(|| env::var("AZ_SYNC_TOKEN").ok())
                .filter(|value| !value.trim().is_empty())
        }

        fn database_url(&self) -> Option<String> {
            self.database_url
                .clone()
                .or_else(|| env::var("AZ_SYNC_DATABASE_URL").ok())
                .filter(|value| !value.trim().is_empty())
        }

        fn object_store_config(&self) -> Option<SyncCliObjectStoreConfig> {
            sync_object_store_config_from_options(
                self.object_dir.clone(),
                self.object_endpoint.clone(),
                self.object_bucket.clone(),
                self.object_access_key.clone(),
                self.object_secret_key.clone(),
                self.object_region.clone(),
            )
        }
    }

    #[derive(Debug, Eq, PartialEq)]
    enum SyncCliObjectStoreConfig {
        FileSystem { root_dir: PathBuf },
        Rustfs(SyncRustfsObjectStoreConfig),
    }

    impl SyncCliObjectStoreConfig {
        fn build_store(&self) -> SyncCliObjectStore {
            match self {
                Self::FileSystem { root_dir } => SyncCliObjectStore::FileSystem(
                    FileSystemSyncObjectStore::new(SyncFileSystemObjectStoreConfig::new(root_dir)),
                ),
                Self::Rustfs(config) => SyncCliObjectStore::Rustfs(config.build_store()),
            }
        }
    }

    #[derive(Debug, Eq, PartialEq)]
    struct SyncRustfsObjectStoreConfig {
        endpoint: String,
        bucket: String,
        access_key: String,
        secret_key: String,
        region: String,
    }

    impl SyncRustfsObjectStoreConfig {
        fn build_store(&self) -> RustfsSyncObjectStore {
            let client = az_rustfs::create_storage_client(
                S3ClientConfig::new(
                    self.endpoint.clone(),
                    self.access_key.clone(),
                    self.secret_key.clone(),
                )
                .with_region(self.region.clone())
                .with_path_style_access(true),
            );
            RustfsSyncObjectStore::new(client, self.bucket.clone(), SyncObjectStoreConfig::new())
        }
    }

    enum SyncCliObjectStore {
        FileSystem(FileSystemSyncObjectStore),
        Rustfs(RustfsSyncObjectStore),
    }

    impl SyncCliObjectStore {
        fn ensure(&self) -> Result<(), SyncCliError> {
            match self {
                Self::FileSystem(_) => Ok(()),
                Self::Rustfs(store) => store.ensure_bucket().map_err(Into::into),
            }
        }

        fn put_file(
            &self,
            space_id: impl Into<String>,
            relative_path: &str,
            source_path: impl AsRef<Path>,
        ) -> Result<SyncObjectManifest, SyncCliError> {
            match self {
                Self::FileSystem(store) => store
                    .put_file(space_id, relative_path, source_path)
                    .map_err(Into::into),
                Self::Rustfs(store) => store
                    .put_file(space_id, relative_path, source_path)
                    .map_err(Into::into),
            }
        }

        fn materialize_file(
            &self,
            manifest: &SyncObjectManifest,
            target_path: impl AsRef<Path>,
        ) -> Result<(), SyncCliError> {
            match self {
                Self::FileSystem(store) => store
                    .materialize_file(manifest, target_path)
                    .map_err(Into::into),
                Self::Rustfs(store) => store
                    .materialize_file(manifest, target_path)
                    .map_err(Into::into),
            }
        }
    }

    #[derive(Debug, Eq, PartialEq)]
    struct AgentArgs {
        once: bool,
        server: Option<String>,
        token: Option<String>,
        object_dir: Option<PathBuf>,
        object_endpoint: Option<String>,
        object_bucket: Option<String>,
        object_access_key: Option<String>,
        object_secret_key: Option<String>,
        object_region: Option<String>,
        common: CommonAgentArgs,
    }

    impl AgentArgs {
        fn parse(args: Vec<String>) -> Result<Self, SyncCliError> {
            let mut parser = ArgParser::new(args);
            let once = parser.take_flag("--once");
            let server = parser.take_option("--server")?;
            let token = parser.take_option("--token")?;
            let object_dir = parser.take_option("--object-dir")?.map(PathBuf::from);
            let object_endpoint = parser.take_option("--object-endpoint")?;
            let object_bucket = parser.take_option("--object-bucket")?;
            let object_access_key = parser.take_option("--object-access-key")?;
            let object_secret_key = parser.take_option("--object-secret-key")?;
            let object_region = parser.take_option("--object-region")?;
            let common = CommonAgentArgs::parse(&mut parser)?;
            parser.finish()?;
            Ok(Self {
                once,
                server,
                token,
                object_dir,
                object_endpoint,
                object_bucket,
                object_access_key,
                object_secret_key,
                object_region,
                common,
            })
        }

        fn agent_config(&self) -> Result<SyncAgentConfig, SyncCliError> {
            self.common.agent_config()
        }

        fn endpoint(&self) -> Option<String> {
            self.server
                .clone()
                .or_else(|| env::var("AZ_SYNC_SERVER").ok())
                .map(|value| sync_ws_endpoint(&value))
        }

        fn auth_token(&self) -> Option<String> {
            self.token
                .clone()
                .or_else(|| env::var("AZ_SYNC_TOKEN").ok())
                .filter(|value| !value.trim().is_empty())
        }

        fn object_store_config(&self) -> Option<SyncCliObjectStoreConfig> {
            sync_object_store_config_from_options(
                self.object_dir.clone(),
                self.object_endpoint.clone(),
                self.object_bucket.clone(),
                self.object_access_key.clone(),
                self.object_secret_key.clone(),
                self.object_region.clone(),
            )
        }
    }

    #[derive(Debug, Eq, PartialEq)]
    struct RootAddArgs {
        path: String,
        alias: Option<String>,
        space_id: Option<String>,
        device_name: Option<String>,
        home: Option<PathBuf>,
    }

    impl RootAddArgs {
        fn parse(args: Vec<String>) -> Result<Self, SyncCliError> {
            let mut parser = ArgParser::new(args);
            let alias = parser.take_option("--alias")?;
            let space_id = parser.take_option("--space-id")?;
            let device_name = parser.take_option("--device-name")?;
            let home = parser.take_option("--home")?.map(PathBuf::from);
            let path = parser.take_positional("root path")?;
            parser.finish()?;
            Ok(Self {
                path,
                alias,
                space_id,
                device_name,
                home,
            })
        }
    }

    #[derive(Debug, Default, Eq, PartialEq)]
    struct CommonAgentArgs {
        device_name: Option<String>,
        home: Option<PathBuf>,
        roots: Vec<String>,
        no_write_index: bool,
        no_finder_state: bool,
    }

    impl CommonAgentArgs {
        fn parse(parser: &mut ArgParser) -> Result<Self, SyncCliError> {
            let device_name = parser.take_option("--device-name")?;
            let home = parser.take_option("--home")?.map(PathBuf::from);
            let roots = parser.take_repeated_option("--root")?;
            let no_write_index = parser.take_flag("--no-write-index");
            let no_finder_state = parser.take_flag("--no-finder-state");
            Ok(Self {
                device_name,
                home,
                roots,
                no_write_index,
                no_finder_state,
            })
        }

        fn agent_config(&self) -> Result<SyncAgentConfig, SyncCliError> {
            let device = device_from_options(self.device_name.clone(), self.home.clone())?;
            let mut config = SyncAgentConfig::for_device(device.clone());
            for root in &self.roots {
                let relative_path = home_relative_input(&device, root)?;
                config = config.with_root(SyncAgentRoot::new(
                    default_alias_for_relative_path(&relative_path),
                    &relative_path,
                    "main",
                )?);
            }
            config.write_local_index = !self.no_write_index;
            config.write_finder_state = !self.no_finder_state;
            config.merge_persisted_roots().map_err(Into::into)
        }
    }

    #[derive(Debug)]
    struct ArgParser {
        args: Vec<String>,
    }

    impl ArgParser {
        fn new(args: Vec<String>) -> Self {
            Self { args }
        }

        fn take_flag(&mut self, flag: &str) -> bool {
            let Some(index) = self.args.iter().position(|value| value == flag) else {
                return false;
            };
            self.args.remove(index);
            true
        }

        fn take_option(&mut self, flag: &str) -> Result<Option<String>, SyncCliError> {
            let Some(index) = self.args.iter().position(|value| value == flag) else {
                return Ok(None);
            };
            self.args.remove(index);
            let Some(value) = self.args.get(index).cloned() else {
                return Err(SyncCliError::Usage(format!("missing value for `{flag}`")));
            };
            self.args.remove(index);
            Ok(Some(value))
        }

        fn take_repeated_option(&mut self, flag: &str) -> Result<Vec<String>, SyncCliError> {
            let mut values = Vec::new();
            while let Some(value) = self.take_option(flag)? {
                values.push(value);
            }
            Ok(values)
        }

        fn take_positional(&mut self, label: &'static str) -> Result<String, SyncCliError> {
            let Some(index) = self.args.iter().position(|value| !value.starts_with('-')) else {
                return Err(SyncCliError::Usage(format!(
                    "missing {label}\n\n{}",
                    usage()
                )));
            };
            Ok(self.args.remove(index))
        }

        fn finish(self) -> Result<(), SyncCliError> {
            if self.args.is_empty() {
                Ok(())
            } else {
                Err(SyncCliError::Usage(format!(
                    "unknown arguments: {}\n\n{}",
                    self.args.join(" "),
                    usage()
                )))
            }
        }
    }

    fn device_from_options(
        device_name: Option<String>,
        home: Option<PathBuf>,
    ) -> Result<SyncDeviceInfo, SyncCliError> {
        let detected = SyncDeviceInfo::detect();
        let home_dir = home.unwrap_or(detected.home_dir);
        if !home_dir.exists() {
            fs::create_dir_all(&home_dir).map_err(|source| {
                SyncCliError::io_error("create sync home directory", Some(home_dir.clone()), source)
            })?;
        }
        Ok(SyncDeviceInfo::new(
            device_name.unwrap_or(detected.device_name),
            home_dir,
        ))
    }

    fn home_relative_input(device: &SyncDeviceInfo, value: &str) -> Result<String, SyncCliError> {
        let expanded = expand_home(value, &device.home_dir);
        let path = Path::new(&expanded);
        if path.is_absolute() {
            Ok(device.home_relative_path(path)?)
        } else {
            Ok(normalize_home_relative_path(&expanded)?)
        }
    }

    fn expand_home(value: &str, home: &Path) -> String {
        value
            .strip_prefix("~/")
            .map(|relative| home.join(relative).to_string_lossy().to_string())
            .unwrap_or_else(|| value.to_string())
    }

    fn default_alias_for_relative_path(relative_path: &str) -> String {
        relative_path
            .rsplit('/')
            .find(|part| !part.is_empty())
            .unwrap_or("root")
            .trim_start_matches('.')
            .replace([' ', '\\'], "-")
    }

    fn sync_ws_endpoint(value: &str) -> String {
        let trimmed = value.trim().trim_end_matches('/');
        if trimmed.ends_with("/api/sync/ws") {
            return trimmed.to_string();
        }
        let websocket_base = trimmed
            .strip_prefix("https://")
            .map(|rest| format!("wss://{rest}"))
            .or_else(|| {
                trimmed
                    .strip_prefix("http://")
                    .map(|rest| format!("ws://{rest}"))
            })
            .unwrap_or_else(|| trimmed.to_string());
        format!("{websocket_base}/api/sync/ws")
    }

    fn sync_object_store_config_from_options(
        object_dir: Option<PathBuf>,
        object_endpoint: Option<String>,
        object_bucket: Option<String>,
        object_access_key: Option<String>,
        object_secret_key: Option<String>,
        object_region: Option<String>,
    ) -> Option<SyncCliObjectStoreConfig> {
        if let Some(root_dir) = object_dir
            .or_else(|| env::var_os("AZ_SYNC_OBJECT_DIR").map(PathBuf::from))
            .filter(|path| !path.as_os_str().is_empty())
        {
            return Some(SyncCliObjectStoreConfig::FileSystem { root_dir });
        }

        let endpoint = non_empty_option_or_env(object_endpoint, "AZ_SYNC_OBJECT_ENDPOINT")?;
        let bucket = non_empty_option_or_env(object_bucket, "AZ_SYNC_OBJECT_BUCKET")?;
        let access_key = non_empty_option_or_env(object_access_key, "AZ_SYNC_OBJECT_ACCESS_KEY")?;
        let secret_key = non_empty_option_or_env(object_secret_key, "AZ_SYNC_OBJECT_SECRET_KEY")?;
        let region = non_empty_option_or_env(object_region, "AZ_SYNC_OBJECT_REGION")
            .unwrap_or_else(|| "us-east-1".to_string());
        Some(SyncCliObjectStoreConfig::Rustfs(
            SyncRustfsObjectStoreConfig {
                endpoint,
                bucket,
                access_key,
                secret_key,
                region,
            },
        ))
    }

    fn non_empty_option_or_env(value: Option<String>, env_name: &str) -> Option<String> {
        value
            .or_else(|| env::var(env_name).ok())
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    }

    fn print_json(value: &impl Serialize) -> Result<(), SyncCliError> {
        let json = serde_json::to_string_pretty(value).map_err(SyncCliError::Json)?;
        println!("{json}");
        Ok(())
    }

    fn usage() -> String {
        [
            "usage:",
            "  az-aio sync serve [--bind 127.0.0.1:8787] [--token TOKEN] [--database-url POSTGRES_URL] [--object-dir PATH | --object-endpoint URL --object-bucket NAME --object-access-key KEY --object-secret-key SECRET] [--home PATH] [--device-name NAME]",
            "  az-aio sync agent [--once] [--server ws://HOST/api/sync/ws] [--token TOKEN] [--object-dir PATH | --object-endpoint URL --object-bucket NAME --object-access-key KEY --object-secret-key SECRET] [--home PATH] [--device-name NAME] [--root HOME_RELATIVE_PATH]",
            "  az-aio sync root add <HOME_RELATIVE_OR_LOCAL_PATH> [--alias NAME] [--space-id SPACE] [--home PATH]",
        ]
        .join("\n")
    }

    #[derive(Debug, Serialize)]
    struct AgentReport {
        event: String,
        status: az_aio_plugin_sync::contracts::SyncStatusResponse,
    }

    impl AgentReport {
        fn from_engine(
            event: impl Into<String>,
            engine: &SyncEngine,
        ) -> Result<Self, SyncCliError> {
            Ok(Self {
                event: event.into(),
                status: engine.status(),
            })
        }
    }

    #[derive(Debug, Error)]
    pub enum SyncCliError {
        #[error("{0}")]
        Usage(String),
        #[error(transparent)]
        Sync(#[from] SyncError),
        #[error("invalid bind address: {0}")]
        Addr(#[from] std::net::AddrParseError),
        #[error("JSON output failed: {0}")]
        Json(serde_json::Error),
        #[error("PostgreSQL sync repository failed: {0}")]
        Sqlx(#[from] sqlx::Error),
        #[error("{operation} failed{path_text}: {source}")]
        Io {
            operation: &'static str,
            path_text: String,
            path: Option<PathBuf>,
            source: std::io::Error,
        },
        #[error("sync watcher channel failed: {0}")]
        WatchChannel(String),
        #[error(
            "object store is required for binary sync; configure --object-dir/AZ_SYNC_OBJECT_DIR for local tests or AZ_SYNC_OBJECT_* for MinIO/RustFS"
        )]
        ObjectStoreUnavailable,
    }

    impl SyncCliError {
        fn io_error(
            operation: &'static str,
            path: Option<PathBuf>,
            source: std::io::Error,
        ) -> Self {
            let path_text = path
                .as_ref()
                .map(|path| format!(" for `{}`", path.display()))
                .unwrap_or_default();
            Self::Io {
                operation,
                path_text,
                path,
                source,
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use std::{fs, path::PathBuf};

        use super::{
            AgentArgs, ServeArgs, SyncCliCommand, SyncCliObjectStoreConfig,
            collect_existing_binary_paths, default_alias_for_relative_path,
            space_id_for_relative_path, sync_ws_endpoint,
        };

        #[test]
        fn parses_az_aio_sync_serve_shape() -> Result<(), Box<dyn std::error::Error>> {
            let command = SyncCliCommand::parse(vec![
                "sync".to_string(),
                "serve".to_string(),
                "--bind".to_string(),
                "127.0.0.1:0".to_string(),
                "--home".to_string(),
                "/tmp/sync-home".to_string(),
            ])?;

            assert_eq!(
                command,
                SyncCliCommand::Serve(ServeArgs {
                    bind: "127.0.0.1:0".to_string(),
                    token: None,
                    database_url: None,
                    object_dir: None,
                    object_endpoint: None,
                    object_bucket: None,
                    object_access_key: None,
                    object_secret_key: None,
                    object_region: None,
                    common: super::CommonAgentArgs {
                        home: Some(PathBuf::from("/tmp/sync-home")),
                        ..Default::default()
                    },
                })
            );
            Ok(())
        }

        #[test]
        fn parses_az_aio_sync_serve_object_store_shape() -> Result<(), Box<dyn std::error::Error>> {
            let command = SyncCliCommand::parse(vec![
                "sync".to_string(),
                "serve".to_string(),
                "--object-endpoint".to_string(),
                "http://localhost:9000".to_string(),
                "--object-bucket".to_string(),
                "az-sync-objects".to_string(),
                "--object-access-key".to_string(),
                "ak".to_string(),
                "--object-secret-key".to_string(),
                "sk".to_string(),
                "--object-region".to_string(),
                "us-west-2".to_string(),
            ])?;

            assert_eq!(
                command,
                SyncCliCommand::Serve(ServeArgs {
                    bind: "127.0.0.1:8787".to_string(),
                    token: None,
                    database_url: None,
                    object_dir: None,
                    object_endpoint: Some("http://localhost:9000".to_string()),
                    object_bucket: Some("az-sync-objects".to_string()),
                    object_access_key: Some("ak".to_string()),
                    object_secret_key: Some("sk".to_string()),
                    object_region: Some("us-west-2".to_string()),
                    common: super::CommonAgentArgs::default(),
                })
            );
            Ok(())
        }

        #[test]
        fn parses_foreground_agent_with_once_flag() -> Result<(), Box<dyn std::error::Error>> {
            let command = SyncCliCommand::parse(vec![
                "sync".to_string(),
                "agent".to_string(),
                "--once".to_string(),
                "--root".to_string(),
                ".agents/skills".to_string(),
            ])?;

            assert_eq!(
                command,
                SyncCliCommand::Agent(AgentArgs {
                    once: true,
                    server: None,
                    token: None,
                    object_dir: None,
                    object_endpoint: None,
                    object_bucket: None,
                    object_access_key: None,
                    object_secret_key: None,
                    object_region: None,
                    common: super::CommonAgentArgs {
                        roots: vec![".agents/skills".to_string()],
                        ..Default::default()
                    },
                })
            );
            Ok(())
        }

        #[test]
        fn parses_agent_object_dir_for_local_binary_sync() -> Result<(), Box<dyn std::error::Error>>
        {
            let command = SyncCliCommand::parse(vec![
                "sync".to_string(),
                "agent".to_string(),
                "--object-dir".to_string(),
                "/tmp/sync-objects".to_string(),
            ])?;

            let SyncCliCommand::Agent(args) = command else {
                panic!("expected agent command");
            };
            assert_eq!(
                args.object_store_config(),
                Some(SyncCliObjectStoreConfig::FileSystem {
                    root_dir: PathBuf::from("/tmp/sync-objects"),
                })
            );
            Ok(())
        }

        #[test]
        fn derives_stable_root_alias_from_home_relative_path() {
            assert_eq!(default_alias_for_relative_path(".agents/skills"), "skills");
            assert_eq!(default_alias_for_relative_path("az-sync"), "az-sync");
        }

        #[test]
        fn normalizes_sync_server_to_websocket_endpoint() {
            assert_eq!(
                sync_ws_endpoint("https://sync.addzero.site"),
                "wss://sync.addzero.site/api/sync/ws"
            );
            assert_eq!(
                sync_ws_endpoint("ws://127.0.0.1:8787/api/sync/ws"),
                "ws://127.0.0.1:8787/api/sync/ws"
            );
        }

        #[test]
        fn selects_space_id_from_longest_matching_sync_root()
        -> Result<(), Box<dyn std::error::Error>> {
            let device = az_aio_plugin_sync::SyncDeviceInfo::new("mac-a", "/tmp/home-a");
            let mut engine = az_aio_plugin_sync::SyncEngine::with_device(device);
            engine.add_root("skills", ".agents/skills", "skills-space")?;

            assert_eq!(
                space_id_for_relative_path(&engine, ".agents/skills/foo/SKILL.md"),
                "skills-space"
            );
            assert_eq!(space_id_for_relative_path(&engine, "az-sync/a.bin"), "main");
            Ok(())
        }

        #[test]
        fn collects_existing_binary_files_as_home_relative_paths()
        -> Result<(), Box<dyn std::error::Error>> {
            let temp_dir = tempfile::tempdir()?;
            let home_dir = temp_dir.path().join("home-a");
            fs::create_dir_all(home_dir.join("az-sync"))?;
            fs::write(home_dir.join("az-sync/a.txt"), "alpha")?;
            fs::write(home_dir.join("az-sync/blob.bin"), [0, 159, 146, 150])?;
            let engine = az_aio_plugin_sync::SyncEngine::with_device(
                az_aio_plugin_sync::SyncDeviceInfo::new("mac-a", home_dir),
            );

            assert_eq!(
                collect_existing_binary_paths(&engine)?,
                vec!["az-sync/blob.bin"]
            );
            Ok(())
        }
    }
}
