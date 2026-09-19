//! XLT 服务数据模型
//!
//! 定义信路通 MQTT 协议消息类型、停车业务实体和配置结构。
//!
//! 按领域拆分为三个子模块：
//! - [`car`] — 车辆相关（识别、事件、在场车辆等）
//! - [`order`] — 计费/订单相关
//! - [`device`] — 设备/MQTT/配置相关

pub mod car;
pub mod device;
pub mod order;

// Re-export all types for backward compatibility
pub use car::{
    AddCarInfoData, CarInfoData, CarInfoItem, CarRetentionData, DeleteCarInfoData,
    OfflineResultData, ParkingVehicle, QueryCarInfoData, ResultData, RtdData, SnapshotData,
    VehicleDetectionData, VehicleEvent, VehicleQuery,
};
pub use device::{
    BarrierStatusData, ChangeKeyData, ClearDataMsg, ConfigData, ConnData, DeviceAlarmData,
    EnableUpdateData, EncryptDeviceData, EncryptionData, IOStatusData, ImageConfig, LcdItemsData,
    MqttConfig, MqttEnvelope, OverUpdateData, PassRuleData, ReplyData, SerialConfigData,
    SerialDataItem, SerialDataMsg, XltConfig,
};
pub use order::{BillingRequest, BillingResult, BillingRule};
