use std::num::{NonZeroU16, NonZeroU32};
use std::ops::Not;

use serde::ser::SerializeSeq as _;
use serde::{Deserialize, Serialize, Serializer};
use strum_macros::IntoStaticStr;

use crate::message::{OutgoingMessageContents, OutgoingMessageNode};
use crate::{adapter, Announcement, GroupConfigUpdate, MemberInfoUpdate, UserDetails};

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyArgs<'a> {
    pub verify_key: &'a str,
}

#[derive(Clone, Debug, Deserialize)]
pub struct VerifyResult {
    pub session: String,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct BindArgs {
    pub qq: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AboutResult {
    pub version: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct GetSessionInfoResult {
    pub qq: UserDetails,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct TargetArgs {
    pub target: i64,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct NudgeArgs {
    pub target: i64,
    pub subject: i64,
    pub kind: SubjectKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub enum SubjectKind {
    Friend,
    Group,
    Stranger,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct SendMessageArgs<'a> {
    pub target: i64,
    #[serde(flatten)]
    pub contents: &'a OutgoingMessageContents<'a>,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct SendTempMessageArgs<'a> {
    pub qq: i64,
    pub group: i64,
    #[serde(flatten)]
    pub contents: &'a OutgoingMessageContents<'a>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageResult {
    pub message_id: i32,
}

impl<E> From<SendMessageResult> for Result<i32, E>
where
    adapter::Error: Into<E>,
{
    fn from(value: SendMessageResult) -> Self {
        (value.message_id != -1)
            .then_some(value.message_id)
            .ok_or_else(|| {
                adapter::Error {
                    code: NonZeroU16::new(500).unwrap(),
                    message: "message was rejected".to_owned(),
                }
                .into()
            })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, IntoStaticStr)]
pub enum MediaType {
    Friend,
    Group,
    Temp,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageIdArgs {
    pub target: i64,
    pub message_id: i32,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoamingMessagesArgs {
    pub time_start: i64,
    pub time_end: i64,
    #[serde(flatten)]
    pub target: RoamingMessagesTarget,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RoamingMessagesTarget {
    #[serde(rename = "qq")]
    Friend(i64),
    Group(i64),
}

#[derive(Clone, Copy, Debug)]
pub struct HandleNewFriendRequestArgs {
    pub event_id: i64,
    pub from_id: i64,
    pub operation: NewFriendRequestOperation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NewFriendRequestOperation {
    Accept = 0,
    Reject = 1,
    RejectAndBlock = 2,
}

#[derive(Clone, Copy, Debug)]
pub struct HandleMemberJoinRequestArgs<'a> {
    pub event_id: i64,
    pub from_id: i64,
    pub group_id: i64,
    pub operation: MemberJoinRequestOperation,
    pub message: &'a str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MemberJoinRequestOperation {
    Accept = 0,
    Reject = 1,
    Ignore = 2,
    RejectAndBlock = 3,
    IgnoreAndBlock = 4,
}

#[derive(Clone, Copy, Debug)]
pub struct HandleBotInvitedJoinGroupRequestArgs {
    pub event_id: i64,
    pub from_id: i64,
    pub group_id: i64,
    pub operation: BotInvitedJoinGroupRequestOperation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BotInvitedJoinGroupRequestOperation {
    Accept = 0,
    Ignore = 1,
}

const _: () = {
    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Args<'a> {
        event_id: i64,
        from_id: i64,
        group_id: i64,
        operate: i32,
        message: &'a str,
    }

    impl Serialize for HandleNewFriendRequestArgs {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            Args {
                event_id: self.event_id,
                from_id: self.from_id,
                group_id: 0,
                operate: self.operation as _,
                message: "",
            }
            .serialize(serializer)
        }
    }

    impl Serialize for HandleMemberJoinRequestArgs<'_> {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            Args {
                event_id: self.event_id,
                from_id: self.from_id,
                group_id: self.group_id,
                operate: self.operation as _,
                message: self.message,
            }
            .serialize(serializer)
        }
    }

    impl Serialize for HandleBotInvitedJoinGroupRequestArgs {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            Args {
                event_id: self.event_id,
                from_id: self.from_id,
                group_id: self.group_id,
                operate: self.operation as _,
                message: "",
            }
            .serialize(serializer)
        }
    }
};

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MuteArgs {
    pub target: i64,
    pub member_id: i64,
    pub time: i32,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KickArgs<'a> {
    pub target: i64,
    pub member_id: i64,
    #[serde(skip_serializing_if = "Not::not")]
    pub block: bool,
    #[serde(skip_serializing_if = "str::is_empty")]
    pub msg: &'a str,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModifyMemberAdminArgs {
    pub target: i64,
    pub member_id: i64,
    pub assign: bool,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateGroupConfigArgs<'a> {
    pub target: i64,
    pub config: &'a GroupConfigUpdate<'a>,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberArgs {
    pub target: i64,
    pub member_id: i64,
}

#[derive(Clone, Copy, Debug)]
pub struct MultiMemberArgs<'a> {
    pub target: i64,
    pub member_ids: &'a [i64],
}

impl Serialize for MultiMemberArgs<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(1 + self.member_ids.len()))?;
        seq.serialize_element(&("target", self.target))?;
        for id in self.member_ids {
            seq.serialize_element(&("memberIds", id))?;
        }
        seq.end()
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMemberInfoArgs<'a> {
    pub target: i64,
    pub member_id: i64,
    pub info: &'a MemberInfoUpdate<'a>,
}

#[derive(Clone, Copy, Debug)]
pub enum FileLocator<'a> {
    Id(&'a str),
    Path(&'a str),
}

impl FileLocator<'_> {
    pub fn root() -> Self {
        Self::Id("")
    }
}

impl Serialize for FileLocator<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Id(id) => {
                #[derive(Debug, Serialize)]
                pub struct Id<'a> {
                    #[serde(skip_serializing_if = "str::is_empty")]
                    pub id: &'a str,
                }

                Id { id }.serialize(serializer)
            }
            Self::Path(path) => {
                #[derive(Debug, Serialize)]
                pub struct Path<'a> {
                    pub path: &'a str,
                }

                Path { path }.serialize(serializer)
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct FileArgs<'a> {
    #[serde(flatten)]
    pub file: FileLocator<'a>,
    pub target: i64,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListFileArgs<'a> {
    #[serde(flatten)]
    pub directory: FileLocator<'a>,
    pub target: i64,
    #[serde(skip_serializing_if = "is_zero")]
    pub offset: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<i32>,
    #[serde(skip_serializing_if = "Not::not")]
    pub with_download_info: bool,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetFileInfoArgs<'a> {
    #[serde(flatten)]
    pub file: FileLocator<'a>,
    pub target: i64,
    #[serde(skip_serializing_if = "Not::not")]
    pub with_download_info: bool,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MkDirArgs<'a> {
    #[serde(flatten)]
    pub directory: FileLocator<'a>,
    pub target: i64,
    pub directory_name: &'a str,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameFileArgs<'a> {
    #[serde(flatten)]
    pub file: FileLocator<'a>,
    pub target: i64,
    pub rename_to: &'a str,
}

fn serialize_move_to<S: Serializer>(value: &FileLocator, serializer: S) -> Result<S::Ok, S::Error> {
    match value {
        FileLocator::Id(id) => {
            #[derive(Debug, Serialize)]
            pub struct Id<'a> {
                #[serde(rename = "moveTo")]
                pub id: &'a str,
            }

            Id { id }.serialize(serializer)
        }
        FileLocator::Path(path) => {
            #[derive(Debug, Serialize)]
            pub struct Path<'a> {
                #[serde(rename = "moveToPath")]
                pub path: &'a str,
            }

            Path { path }.serialize(serializer)
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct MoveFileArgs<'a> {
    #[serde(flatten)]
    pub file: FileLocator<'a>,
    pub target: i64,
    #[serde(flatten, serialize_with = "serialize_move_to")]
    pub move_to: FileLocator<'a>,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct ExecuteCommandArgs<'a> {
    pub command: &'a [OutgoingMessageNode<'a>],
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct ListAnnouncementArgs {
    pub id: i64,
    #[serde(skip_serializing_if = "is_zero")]
    pub offset: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<i32>,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct AnnouncementArgs<'a> {
    pub id: i64,
    pub fid: &'a str,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct PublishAnnouncementArgs<'a> {
    pub target: i64,
    #[serde(flatten)]
    pub announcement: &'a Announcement<'a>,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct CountArgs {
    pub count: Option<NonZeroU32>,
}

fn is_zero(value: &i32) -> bool {
    *value == 0
}
