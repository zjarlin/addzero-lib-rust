# az-git

`az-git` provides local Git hosting account discovery and login metadata for
desktop applications.

The first implementation supports GitHub, Gitee, and GitLab. GitHub can reuse
the local `gh` CLI login state when it is present. Web login and token login are
represented as provider-neutral flows so application shells can render their own
UI without embedding provider-specific strings.
