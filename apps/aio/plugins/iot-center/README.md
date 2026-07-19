# az-aio-iot-center

AIO 物联网中心子插件。插件使用 `az-engine` 动态模型承载产品、网关、设备、遥测和告警，
正式数据统一保存到共享 PostgreSQL。

设备业务状态不等同于 MQTT 连接状态：页面分别检查连接、心跳和数据新鲜度，展示
`Online`、`HeartbeatLost`、`DataAnomaly`、`Offline`、`Unknown` 五种状态。

初始化接口：`POST /api/iot/templates/default/apply`。
