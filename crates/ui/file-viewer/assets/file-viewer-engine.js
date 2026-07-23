(() => {
  const selector = "[data-file-viewer-engine]";
  const rootStates = new WeakMap();
  const scriptLoads = new Map();
  const moduleLoads = new Map();

  function loadScript(url, isReady) {
    if (isReady()) {
      return Promise.resolve();
    }
    if (scriptLoads.has(url)) {
      return scriptLoads.get(url);
    }
    const promise = new Promise((resolve, reject) => {
      const script = document.createElement("script");
      script.src = url;
      script.onload = () => {
        if (isReady()) {
          resolve();
          return;
        }
        reject(new Error(`脚本加载后未提供预期能力：${url}`));
      };
      script.onerror = () => reject(new Error(`脚本加载失败：${url}`));
      document.head.appendChild(script);
    });
    scriptLoads.set(url, promise);
    return promise;
  }

  function loadModule(url) {
    if (!moduleLoads.has(url)) {
      moduleLoads.set(url, import(url));
    }
    return moduleLoads.get(url);
  }

  function readConfig(root) {
    return {
      kind: root.dataset.kind || "",
      name: root.dataset.name || "",
      src: root.dataset.src || "",
      markedScript: root.dataset.markedScript || "",
      dompurifyScript: root.dataset.dompurifyScript || "",
      pdfModule: root.dataset.pdfModule || "",
      pdfWorker: root.dataset.pdfWorker || "",
      jszipScript: root.dataset.jszipScript || "",
      docxScript: root.dataset.docxScript || "",
    };
  }

  function beginRender(root) {
    const previous = rootStates.get(root);
    if (previous?.dispose) {
      previous.dispose();
    }
    const state = { dispose: null };
    rootStates.set(root, state);
    return state;
  }

  function isCurrent(root, state) {
    return rootStates.get(root) === state;
  }

  function showMessage(root, message, isError = false) {
    const node = document.createElement("p");
    node.className = isError
      ? "file-viewer__message file-viewer__message--error"
      : "file-viewer__message";
    node.textContent = message;
    root.replaceChildren(node);
  }

  async function fetchSource(src, mode) {
    const response = await fetch(src);
    if (!response.ok) {
      throw new Error(`文件请求失败：HTTP ${response.status}`);
    }
    if (mode === "text") {
      return response.text();
    }
    return response.arrayBuffer();
  }

  async function renderMarkdown(root, config, state) {
    await Promise.all([
      loadScript(config.markedScript, () => Boolean(globalThis.marked?.parse)),
      loadScript(config.dompurifyScript, () => Boolean(globalThis.DOMPurify?.sanitize)),
    ]);
    const text = await fetchSource(config.src, "text");
    if (!isCurrent(root, state)) {
      return;
    }
    const rendered = globalThis.marked.parse(text, { gfm: true });
    const sanitized = globalThis.DOMPurify.sanitize(rendered, {
      USE_PROFILES: { html: true },
    });
    const article = document.createElement("article");
    article.className = "file-viewer__markdown";
    article.innerHTML = sanitized;
    root.replaceChildren(article);
  }

  async function renderText(root, config, state) {
    const text = await fetchSource(config.src, "text");
    if (!isCurrent(root, state)) {
      return;
    }
    const pre = document.createElement("pre");
    pre.className = "file-viewer__text";
    pre.textContent = text;
    root.replaceChildren(pre);
  }

  async function renderPdf(root, config, state) {
    const pdfjs = await loadModule(config.pdfModule);
    pdfjs.GlobalWorkerOptions.workerSrc = config.pdfWorker;
    const data = await fetchSource(config.src, "bytes");
    if (!isCurrent(root, state)) {
      return;
    }
    const documentProxy = await pdfjs.getDocument({ data: new Uint8Array(data) }).promise;
    if (!isCurrent(root, state)) {
      await documentProxy.destroy();
      return;
    }

    const pages = document.createElement("div");
    pages.className = "file-viewer__pdf-pages";
    root.replaceChildren(pages);

    let resizeTimer;
    let renderSequence = 0;

    async function renderPages() {
      const sequence = ++renderSequence;
      pages.replaceChildren();
      const width = Math.max(160, root.clientWidth - 32);
      const pixelRatio = Math.min(globalThis.devicePixelRatio || 1, 2);

      for (let pageNumber = 1; pageNumber <= documentProxy.numPages; pageNumber += 1) {
        if (!isCurrent(root, state) || sequence !== renderSequence) {
          return;
        }
        const page = await documentProxy.getPage(pageNumber);
        const baseViewport = page.getViewport({ scale: 1 });
        const cssScale = width / baseViewport.width;
        const viewport = page.getViewport({ scale: cssScale * pixelRatio });
        const canvas = document.createElement("canvas");
        canvas.className = "file-viewer__pdf-page";
        canvas.width = Math.ceil(viewport.width);
        canvas.height = Math.ceil(viewport.height);
        canvas.style.width = `${Math.round(width)}px`;
        canvas.style.height = `${Math.round(width * (baseViewport.height / baseViewport.width))}px`;
        pages.appendChild(canvas);
        await page.render({ canvas, viewport }).promise;
      }
    }

    await renderPages();
    const resizeObserver = new ResizeObserver(() => {
      clearTimeout(resizeTimer);
      resizeTimer = setTimeout(() => void renderPages(), 180);
    });
    resizeObserver.observe(root);
    state.dispose = () => {
      clearTimeout(resizeTimer);
      renderSequence += 1;
      resizeObserver.disconnect();
      void documentProxy.destroy();
    };
  }

  async function renderDocx(root, config, state) {
    await loadScript(config.jszipScript, () => Boolean(globalThis.JSZip));
    await loadScript(config.docxScript, () => Boolean(globalThis.docx?.renderAsync));
    const data = await fetchSource(config.src, "bytes");
    if (!isCurrent(root, state)) {
      return;
    }
    const documentRoot = document.createElement("div");
    documentRoot.className = "file-viewer__docx";
    root.replaceChildren(documentRoot);
    await globalThis.docx.renderAsync(data, documentRoot, undefined, {
      className: "docx",
      inWrapper: true,
      ignoreWidth: true,
      ignoreHeight: true,
      breakPages: true,
      renderHeaders: true,
      renderFooters: true,
      renderFootnotes: true,
      renderEndnotes: true,
      useBase64URL: true,
    });
  }

  async function renderRoot(root) {
    const state = beginRender(root);
    const config = readConfig(root);
    showMessage(root, "正在加载预览…");
    try {
      if (!config.src) {
        throw new Error("未提供文件内容");
      }
      switch (config.kind) {
        case "markdown":
          await renderMarkdown(root, config, state);
          break;
        case "pdf":
          await renderPdf(root, config, state);
          break;
        case "docx":
          await renderDocx(root, config, state);
          break;
        case "text":
          await renderText(root, config, state);
          break;
        default:
          throw new Error(`不支持的预览类型：${config.kind}`);
      }
    } catch (error) {
      if (isCurrent(root, state)) {
        showMessage(root, `预览失败：${String(error)}`, true);
      }
    }
  }

  function mountTree(node) {
    if (!(node instanceof Element)) {
      return;
    }
    if (node.matches(selector)) {
      void renderRoot(node);
    }
    node.querySelectorAll(selector).forEach((root) => void renderRoot(root));
  }

  function disposeTree(node) {
    if (!(node instanceof Element)) {
      return;
    }
    const roots = node.matches(selector)
      ? [node, ...node.querySelectorAll(selector)]
      : [...node.querySelectorAll(selector)];
    roots.forEach((root) => {
      const state = rootStates.get(root);
      if (state?.dispose) {
        state.dispose();
      }
      rootStates.delete(root);
    });
  }

  const observer = new MutationObserver((records) => {
    records.forEach((record) => {
      if (record.type === "attributes") {
        void renderRoot(record.target);
        return;
      }
      record.removedNodes.forEach(disposeTree);
      record.addedNodes.forEach(mountTree);
    });
  });

  observer.observe(document.documentElement, {
    attributes: true,
    attributeFilter: [
      "data-kind",
      "data-name",
      "data-src",
      "data-marked-script",
      "data-dompurify-script",
      "data-pdf-module",
      "data-pdf-worker",
      "data-jszip-script",
      "data-docx-script",
    ],
    childList: true,
    subtree: true,
  });
  mountTree(document.documentElement);
})();
