use bitflags::bitflags;
use serde::{Deserialize, Serialize};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Permission: u8 {
        const SHARE = 1;

        const WRITE_OWNED = 1 << 1;

        const READ_ALL = 1 << 2;
        const WRITE_ALL = 1 << 3;

        const READ_USERS = 1 << 4;
        const WRITE_USERS = 1 << 5;

        const ADMIN = Self::SHARE.bits()
        | Self::WRITE_OWNED.bits()
        | Self::READ_ALL.bits()
        | Self::WRITE_ALL.bits()
        | Self::READ_USERS.bits()
        | Self::WRITE_USERS.bits();

        const UNPRIVILEGED = Self::SHARE.bits()
        | Self::WRITE_OWNED.bits()
        | Self::READ_USERS.bits();

        const SINGLE_FILE_R = 0;
        const SINGLE_FILE_RW = Self::WRITE_OWNED.bits();
    }
}
