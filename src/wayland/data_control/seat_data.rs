use std::{
    os::unix::io::{AsRawFd, OwnedFd},
    sync::Arc,
};

use tracing::debug;
use wayland_protocols_wlr::data_control::v1::server::{
    zwlr_data_control_device_v1::ZwlrDataControlDeviceV1 as Device,
    zwlr_data_control_offer_v1::{self as offer, ZwlrDataControlOfferV1 as offer},
    zwlr_data_control_source_v1::ZwlrDataControlSourceV1 as Source,
};
use wayland_server::{
    backend::{protocol::Message, ClientId, Handle, ObjectData, ObjectId},
    Client, DisplayHandle, Resource,
};

use crate::utils::IsAlive;

use super::{with_source_metadata, Handler, Metadata};
