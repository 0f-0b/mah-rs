#![forbid(unsafe_code)]

pub mod adapter;
pub mod event;
pub mod message;
pub mod types;

use std::borrow::Cow;
use std::ops::Not;
use std::time::{Duration, SystemTime};

use async_trait::async_trait;
use derive_into_owned::IntoOwned;
use serde::{Deserialize, Deserializer, Serialize};
use types::{RoamingMessagesArgs, RoamingMessagesTarget};

use self::adapter::{Bytes, MahSession};
use self::message::{Message, OutgoingMessageContents, OutgoingMessageNode};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MemberPermission {
    Member,
    #[serde(rename = "ADMINISTRATOR")]
    Admin,
    Owner,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Sex {
    Male,
    Female,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum GroupHonor {
    #[serde(rename = "龙王")]
    Talkative,
    #[serde(rename = "群聊之火")]
    Performer,
    #[serde(rename = "群聊炽焰")]
    Legend,
    #[serde(rename = "冒尖小春笋")]
    Emotion,
    #[serde(rename = "快乐源泉")]
    Bronze,
    #[serde(rename = "学术新星")]
    Silver,
    #[serde(rename = "至尊学神")]
    Golden,
    #[serde(rename = "一笔当先")]
    Whirlwind,
    #[serde(rename = "壕礼皇冠")]
    Richer,
    #[serde(rename = "善财福禄寿")]
    RedPacket,
    #[serde(rename = "未知群荣誉")]
    Unknown,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Profile {
    pub nickname: String,
    pub email: String,
    pub age: i32,
    pub level: i32,
    pub sign: String,
    pub sex: Sex,
}

#[derive(Clone, Debug)]
pub enum FileUpload {
    Url(Cow<'static, str>),
    Bytes(Bytes),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageInfo {
    pub image_id: String,
    pub url: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceInfo {
    pub voice_id: String,
}

#[derive(Clone, Debug)]
pub struct FileMetadata {
    pub size: i64,
    pub sha1: String,
    pub md5: String,
    pub uploader_id: i64,
    pub upload_time_secs: i64,
    pub last_modify_time_secs: i64,
    pub download_info: Option<FileDownloadInfo>,
}

impl FileMetadata {
    pub fn uploader(&self, group: GroupHandle) -> MemberHandle {
        group.get_member(self.uploader_id)
    }

    pub fn upload_time(&self) -> Option<SystemTime> {
        SystemTime::UNIX_EPOCH.checked_add(Duration::from_secs(self.upload_time_secs as u64))
    }

    pub fn last_modify_time(&self) -> Option<SystemTime> {
        SystemTime::UNIX_EPOCH.checked_add(Duration::from_secs(self.last_modify_time_secs as u64))
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct FileDownloadInfo {
    pub url: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupConfig {
    pub name: String,
    pub confess_talk: bool,
    pub allow_member_invite: bool,
    pub auto_approve: bool,
    pub anonymous_chat: bool,
    pub mute_all: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MemberActivity {
    pub rank: i32,
    #[serde(rename = "point")]
    pub points: i32,
    pub honors: Vec<GroupHonor>,
    pub temperature: i32,
}

#[derive(Clone, Debug, IntoOwned, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupConfigUpdate<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_member_invite: Option<bool>,
}

impl<'a> GroupConfigUpdate<'a> {
    pub fn new() -> Self {
        Self {
            name: None,
            allow_member_invite: None,
        }
    }

    pub fn name(self, name: Option<impl Into<Cow<'a, str>>>) -> Self {
        Self {
            name: name.map(Into::into),
            ..self
        }
    }

    pub fn allow_member_invite(self, allow_member_invite: Option<bool>) -> Self {
        Self {
            allow_member_invite,
            ..self
        }
    }
}

impl Default for GroupConfigUpdate<'_> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, IntoOwned, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberInfoUpdate<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub special_title: Option<Cow<'a, str>>,
}

impl<'a> MemberInfoUpdate<'a> {
    pub fn new() -> Self {
        Self {
            name: None,
            special_title: None,
        }
    }

    pub fn name(self, name: Option<impl Into<Cow<'a, str>>>) -> Self {
        Self {
            name: name.map(Into::into),
            ..self
        }
    }

    pub fn special_title(self, special_title: Option<impl Into<Cow<'a, str>>>) -> Self {
        Self {
            special_title: special_title.map(Into::into),
            ..self
        }
    }
}

impl Default for MemberInfoUpdate<'_> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, IntoOwned, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Announcement<'a> {
    #[serde(rename = "content")]
    pub contents: Cow<'a, str>,
    #[serde(skip_serializing_if = "Not::not")]
    pub send_to_new_member: bool,
    #[serde(skip_serializing_if = "Not::not")]
    pub pinned: bool,
    #[serde(skip_serializing_if = "Not::not")]
    pub show_edit_card: bool,
    #[serde(skip_serializing_if = "Not::not")]
    pub show_popup: bool,
    #[serde(skip_serializing_if = "Not::not")]
    pub require_confirmation: bool,
    #[serde(flatten)]
    pub image: Option<AnnouncementImage<'a>>,
}

impl<'a> Announcement<'a> {
    pub fn new(contents: impl Into<Cow<'a, str>>) -> Self {
        Self {
            contents: contents.into(),
            send_to_new_member: false,
            pinned: false,
            show_edit_card: false,
            show_popup: false,
            require_confirmation: false,
            image: None,
        }
    }

    pub fn send_to_new_member(self, send_to_new_member: bool) -> Self {
        Self {
            send_to_new_member,
            ..self
        }
    }

    pub fn pinned(self, pinned: bool) -> Self {
        Self { pinned, ..self }
    }

    pub fn show_edit_card(self, show_edit_card: bool) -> Self {
        Self {
            show_edit_card,
            ..self
        }
    }

    pub fn show_popup(self, show_popup: bool) -> Self {
        Self { show_popup, ..self }
    }

    pub fn require_confirmation(self, require_confirmation: bool) -> Self {
        Self {
            require_confirmation,
            ..self
        }
    }

    pub fn image(self, image: Option<impl Into<AnnouncementImage<'a>>>) -> Self {
        Self {
            image: image.map(Into::into),
            ..self
        }
    }
}

#[derive(Clone, Debug, IntoOwned, Serialize)]
pub enum AnnouncementImage<'a> {
    #[serde(rename = "imageUrl")]
    Url(Cow<'a, str>),
    #[serde(rename = "imagePath")]
    Path(Cow<'a, str>),
    #[serde(rename = "imageBase64")]
    Base64(Cow<'a, str>),
}

impl<'a> From<&'a AnnouncementImage<'a>> for AnnouncementImage<'a> {
    fn from(value: &'a AnnouncementImage<'a>) -> Self {
        match value {
            Self::Url(url) => Self::Url(Cow::Borrowed(url)),
            Self::Path(path) => Self::Path(Cow::Borrowed(path)),
            Self::Base64(base64) => Self::Base64(Cow::Borrowed(base64)),
        }
    }
}

#[derive(Clone, Debug, IntoOwned, Serialize)]
pub struct Command<'a> {
    pub name: Cow<'a, str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub alias: Vec<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Cow<'a, str>>,
}

impl<'a> Command<'a> {
    pub fn new(name: impl Into<Cow<'a, str>>) -> Self {
        Self {
            name: name.into(),
            alias: Vec::new(),
            usage: None,
            description: None,
        }
    }

    pub fn alias(self, alias: impl IntoIterator<Item = impl Into<Cow<'a, str>>>) -> Self {
        Self {
            alias: alias.into_iter().map(Into::into).collect(),
            ..self
        }
    }

    pub fn usage(self, usage: Option<impl Into<Cow<'a, str>>>) -> Self {
        Self {
            usage: usage.map(Into::into),
            ..self
        }
    }

    pub fn description(self, description: Option<impl Into<Cow<'a, str>>>) -> Self {
        Self {
            description: description.map(Into::into),
            ..self
        }
    }
}

#[async_trait]
pub trait SendMessage {
    async fn send_message<'a, S: MahSession + ?Sized>(
        &self,
        session: &S,
        message: &'a OutgoingMessageContents<'a>,
    ) -> Result<MessageHandle, S::Error>;
    async fn upload_image<S: MahSession + ?Sized>(
        &self,
        session: &S,
        image: FileUpload,
    ) -> Result<ImageInfo, S::Error>;
    async fn upload_voice<S: MahSession + ?Sized>(
        &self,
        session: &S,
        voice: FileUpload,
    ) -> Result<VoiceInfo, S::Error>;
}

#[async_trait]
pub trait SendNudge {
    async fn send_nudge<S: MahSession + ?Sized>(
        &self,
        session: &S,
        target: UserHandle,
    ) -> Result<(), S::Error>;

    async fn send_nudge_any<S: MahSession + ?Sized>(
        &self,
        session: &S,
        target: impl AnyUserHandle + Send,
    ) -> Result<(), S::Error> {
        let target = target.to_user();
        self.send_nudge(session, target).await
    }
}

#[async_trait]
pub trait GetRoamingMessages {
    async fn get_roaming_messages<S: MahSession + ?Sized>(
        &self,
        session: &S,
        start_time_secs: i64,
        end_time_secs: i64,
    ) -> Result<Vec<Message>, S::Error>;
}

#[async_trait]
pub trait GetProfile {
    async fn get_profile<S: MahSession + ?Sized>(&self, session: &S) -> Result<Profile, S::Error>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Bot;

impl Bot {
    pub fn get_user(&self, id: i64) -> UserHandle {
        UserHandle { id }
    }

    pub fn get_friend(&self, id: i64) -> FriendHandle {
        FriendHandle { id }
    }

    pub fn get_stranger(&self, id: i64) -> StrangerHandle {
        StrangerHandle { id }
    }

    pub fn get_group(&self, id: i64) -> GroupHandle {
        GroupHandle { id }
    }

    pub fn get_other_client(&self, id: i64) -> OtherClientHandle {
        OtherClientHandle { id }
    }

    pub fn get_message(&self, id: i32, context: i64) -> MessageHandle {
        MessageHandle { id, context }
    }

    pub async fn to_user<S: MahSession + ?Sized>(
        &self,
        session: &S,
    ) -> Result<UserDetails, S::Error> {
        Ok(session.get_session_info().await?.qq)
    }

    pub async fn to_friend<S: MahSession + ?Sized>(
        &self,
        session: &S,
    ) -> Result<FriendDetails, S::Error> {
        Ok(FriendDetails(self.to_user(session).await?))
    }

    pub async fn to_stranger<S: MahSession + ?Sized>(
        &self,
        session: &S,
    ) -> Result<StrangerDetails, S::Error> {
        Ok(StrangerDetails(self.to_user(session).await?))
    }

    pub async fn get_friends<S: MahSession + ?Sized>(
        &self,
        session: &S,
    ) -> Result<Vec<FriendDetails>, S::Error> {
        session.get_friend_list().await
    }

    pub async fn get_groups<S: MahSession + ?Sized>(
        &self,
        session: &S,
    ) -> Result<Vec<GroupDetails>, S::Error> {
        session.get_group_list().await
    }

    pub async fn get_profile<S: MahSession + ?Sized>(
        &self,
        session: &S,
    ) -> Result<Profile, S::Error> {
        session.get_bot_profile().await
    }

    pub async fn execute_command<'a, S: MahSession + ?Sized>(
        &self,
        session: &S,
        command: &'a [OutgoingMessageNode<'a>],
    ) -> Result<(), S::Error> {
        session
            .execute_command(&types::ExecuteCommandArgs { command })
            .await
    }

    pub async fn register_command<'a, S: MahSession + ?Sized>(
        &self,
        session: &S,
        command: &'a Command<'a>,
    ) -> Result<(), S::Error> {
        session.register_command(command).await
    }
}

#[async_trait]
impl GetProfile for Bot {
    async fn get_profile<S: MahSession + ?Sized>(&self, session: &S) -> Result<Profile, S::Error> {
        self.get_profile(session).await
    }
}

pub trait AnyUserHandle {
    fn id(&self) -> i64;

    fn to_user(&self) -> UserHandle {
        UserHandle { id: self.id() }
    }

    fn to_friend(&self) -> FriendHandle {
        FriendHandle { id: self.id() }
    }

    fn to_stranger(&self) -> StrangerHandle {
        StrangerHandle { id: self.id() }
    }

    fn to_member(&self, group: GroupHandle) -> MemberHandle {
        MemberHandle {
            id: self.id(),
            group,
        }
    }

    fn avatar_url(&self) -> String {
        format!("https://q1.qlogo.cn/g?b=qq&nk={}&s=640", self.id())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct UserHandle {
    id: i64,
}

impl UserHandle {
    pub async fn get_profile<S: MahSession + ?Sized>(
        &self,
        session: &S,
    ) -> Result<Profile, S::Error> {
        session
            .get_user_profile(&types::TargetArgs { target: self.id })
            .await
    }
}

impl AnyUserHandle for UserHandle {
    fn id(&self) -> i64 {
        self.id
    }
}

#[async_trait]
impl GetProfile for UserHandle {
    async fn get_profile<S: MahSession + ?Sized>(&self, session: &S) -> Result<Profile, S::Error> {
        self.get_profile(session).await
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct UserDetails {
    pub id: i64,
    pub nickname: String,
    pub remark: String,
}

impl UserDetails {
    pub fn handle(&self) -> UserHandle {
        UserHandle { id: self.id }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FriendHandle {
    id: i64,
}

impl FriendHandle {
    pub async fn send_message<'a, S: MahSession + ?Sized>(
        &self,
        session: &S,
        message: &'a OutgoingMessageContents<'a>,
    ) -> Result<MessageHandle, S::Error> {
        Ok(MessageHandle {
            id: session
                .send_friend_message(&types::SendMessageArgs {
                    target: self.id,
                    contents: message,
                })
                .await?,
            context: self.id,
        })
    }

    pub async fn upload_image<S: MahSession + ?Sized>(
        &self,
        session: &S,
        image: FileUpload,
    ) -> Result<ImageInfo, S::Error> {
        session.upload_image(types::MediaType::Friend, image).await
    }

    pub async fn upload_voice<S: MahSession + ?Sized>(
        &self,
        session: &S,
        voice: FileUpload,
    ) -> Result<VoiceInfo, S::Error> {
        session.upload_voice(types::MediaType::Friend, voice).await
    }

    pub async fn send_nudge<S: MahSession + ?Sized>(
        &self,
        session: &S,
        target: UserHandle,
    ) -> Result<(), S::Error> {
        session
            .nudge(&types::NudgeArgs {
                target: target.id,
                subject: self.id,
                kind: types::SubjectKind::Friend,
            })
            .await
    }

    pub async fn get_roaming_messages<S: MahSession + ?Sized>(
        &self,
        session: &S,
        start_time_secs: i64,
        end_time_secs: i64,
    ) -> Result<Vec<Message>, S::Error> {
        session
            .roaming_messages(&RoamingMessagesArgs {
                time_start: start_time_secs,
                time_end: end_time_secs,
                target: RoamingMessagesTarget::Friend(self.id),
            })
            .await
    }

    pub async fn get_profile<S: MahSession + ?Sized>(
        &self,
        session: &S,
    ) -> Result<Profile, S::Error> {
        session
            .get_friend_profile(&types::TargetArgs { target: self.id })
            .await
    }

    pub async fn remove_friend<S: MahSession + ?Sized>(&self, session: &S) -> Result<(), S::Error> {
        session
            .delete_friend(&types::TargetArgs { target: self.id })
            .await
    }
}

impl AnyUserHandle for FriendHandle {
    fn id(&self) -> i64 {
        self.id
    }
}

#[async_trait]
impl SendMessage for FriendHandle {
    async fn send_message<'a, S: MahSession + ?Sized>(
        &self,
        session: &S,
        message: &'a OutgoingMessageContents<'a>,
    ) -> Result<MessageHandle, S::Error> {
        self.send_message(session, message).await
    }

    async fn upload_image<S: MahSession + ?Sized>(
        &self,
        session: &S,
        image: FileUpload,
    ) -> Result<ImageInfo, S::Error> {
        self.upload_image(session, image).await
    }

    async fn upload_voice<S: MahSession + ?Sized>(
        &self,
        session: &S,
        voice: FileUpload,
    ) -> Result<VoiceInfo, S::Error> {
        self.upload_voice(session, voice).await
    }
}

#[async_trait]
impl SendNudge for FriendHandle {
    async fn send_nudge<S: MahSession + ?Sized>(
        &self,
        session: &S,
        target: UserHandle,
    ) -> Result<(), S::Error> {
        self.send_nudge(session, target).await
    }
}

#[async_trait]
impl GetRoamingMessages for FriendHandle {
    async fn get_roaming_messages<S: MahSession + ?Sized>(
        &self,
        session: &S,
        start_time_secs: i64,
        end_time_secs: i64,
    ) -> Result<Vec<Message>, S::Error> {
        self.get_roaming_messages(session, start_time_secs, end_time_secs)
            .await
    }
}

#[async_trait]
impl GetProfile for FriendHandle {
    async fn get_profile<S: MahSession + ?Sized>(&self, session: &S) -> Result<Profile, S::Error> {
        self.get_profile(session).await
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct FriendDetails(pub UserDetails);

impl FriendDetails {
    pub fn handle(&self) -> FriendHandle {
        FriendHandle { id: self.0.id }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct StrangerHandle {
    id: i64,
}

impl StrangerHandle {
    pub async fn send_nudge<S: MahSession + ?Sized>(
        &self,
        session: &S,
        target: UserHandle,
    ) -> Result<(), S::Error> {
        session
            .nudge(&types::NudgeArgs {
                target: target.id,
                subject: self.id,
                kind: types::SubjectKind::Stranger,
            })
            .await
    }

    pub async fn get_profile<S: MahSession + ?Sized>(
        &self,
        session: &S,
    ) -> Result<Profile, S::Error> {
        self.to_user().get_profile(session).await
    }
}

impl AnyUserHandle for StrangerHandle {
    fn id(&self) -> i64 {
        self.id
    }
}

#[async_trait]
impl SendNudge for StrangerHandle {
    async fn send_nudge<S: MahSession + ?Sized>(
        &self,
        session: &S,
        target: UserHandle,
    ) -> Result<(), S::Error> {
        self.send_nudge(session, target).await
    }
}

#[async_trait]
impl GetProfile for StrangerHandle {
    async fn get_profile<S: MahSession + ?Sized>(&self, session: &S) -> Result<Profile, S::Error> {
        self.get_profile(session).await
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct StrangerDetails(pub UserDetails);

impl StrangerDetails {
    pub fn handle(&self) -> StrangerHandle {
        StrangerHandle { id: self.0.id }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GroupHandle {
    id: i64,
}

impl GroupHandle {
    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn get_member(&self, id: i64) -> MemberHandle {
        MemberHandle { id, group: *self }
    }

    pub fn get_file(&self, id: String) -> FileHandle {
        FileHandle { id, group: *self }
    }

    pub fn get_files_root(&self) -> FileHandle {
        const ROOT_ID: String = String::new();
        self.get_file(ROOT_ID)
    }

    pub fn get_announcement(&self, id: String) -> AnnouncementHandle {
        AnnouncementHandle { id, group: *self }
    }

    pub async fn get_members<S: MahSession + ?Sized>(
        &self,
        session: &S,
    ) -> Result<Vec<MemberDetails>, S::Error> {
        session
            .get_member_list(&types::TargetArgs { target: self.id })
            .await
    }

    pub async fn refresh_members<S: MahSession + ?Sized>(
        &self,
        session: &S,
        ids: Option<&[i64]>,
    ) -> Result<Vec<MemberDetails>, S::Error> {
        session
            .latest_member_list(&types::MultiMemberArgs {
                target: self.id,
                member_ids: ids.unwrap_or_default(),
            })
            .await
    }

    pub async fn send_message<'a, S: MahSession + ?Sized>(
        &self,
        session: &S,
        message: &'a OutgoingMessageContents<'a>,
    ) -> Result<MessageHandle, S::Error> {
        Ok(MessageHandle {
            id: session
                .send_group_message(&types::SendMessageArgs {
                    target: self.id,
                    contents: message,
                })
                .await?,
            context: self.id,
        })
    }

    pub async fn upload_image<S: MahSession + ?Sized>(
        &self,
        session: &S,
        image: FileUpload,
    ) -> Result<ImageInfo, S::Error> {
        session.upload_image(types::MediaType::Group, image).await
    }

    pub async fn upload_voice<S: MahSession + ?Sized>(
        &self,
        session: &S,
        voice: FileUpload,
    ) -> Result<VoiceInfo, S::Error> {
        session.upload_voice(types::MediaType::Group, voice).await
    }

    pub async fn send_nudge<S: MahSession + ?Sized>(
        &self,
        session: &S,
        target: UserHandle,
    ) -> Result<(), S::Error> {
        session
            .nudge(&types::NudgeArgs {
                target: target.id,
                subject: self.id,
                kind: types::SubjectKind::Group,
            })
            .await
    }

    pub async fn get_roaming_messages<S: MahSession + ?Sized>(
        &self,
        session: &S,
        start_time_secs: i64,
        end_time_secs: i64,
    ) -> Result<Vec<Message>, S::Error> {
        session
            .roaming_messages(&RoamingMessagesArgs {
                time_start: start_time_secs,
                time_end: end_time_secs,
                target: RoamingMessagesTarget::Group(self.id),
            })
            .await
    }

    pub async fn mute_all<S: MahSession + ?Sized>(&self, session: &S) -> Result<(), S::Error> {
        session
            .mute_all(&types::TargetArgs { target: self.id })
            .await
    }

    pub async fn unmute_all<S: MahSession + ?Sized>(&self, session: &S) -> Result<(), S::Error> {
        session
            .unmute_all(&types::TargetArgs { target: self.id })
            .await
    }

    pub async fn quit<S: MahSession + ?Sized>(&self, session: &S) -> Result<(), S::Error> {
        session.quit(&types::TargetArgs { target: self.id }).await
    }

    pub async fn get_group_config<S: MahSession + ?Sized>(
        &self,
        session: &S,
    ) -> Result<GroupConfig, S::Error> {
        session
            .get_group_config(&types::TargetArgs { target: self.id })
            .await
    }

    pub async fn update_group_config<'a, S: MahSession + ?Sized>(
        &self,
        session: &S,
        config: &'a GroupConfigUpdate<'a>,
    ) -> Result<(), S::Error> {
        session
            .update_group_config(&types::UpdateGroupConfigArgs {
                target: self.id,
                config,
            })
            .await
    }

    pub async fn list_files<S: MahSession + ?Sized>(
        &self,
        session: &S,
        path: Option<&str>,
        range: (i32, Option<i32>),
        download: bool,
    ) -> Result<Vec<FileDetails>, S::Error> {
        session
            .list_file(&types::ListFileArgs {
                directory: path.map_or_else(types::FileLocator::root, types::FileLocator::Path),
                target: self.id,
                offset: range.0,
                size: range.1,
                with_download_info: download,
            })
            .await
    }

    pub async fn get_file_info<S: MahSession + ?Sized>(
        &self,
        session: &S,
        path: &str,
        download: bool,
    ) -> Result<FileDetails, S::Error> {
        session
            .get_file_info(&types::GetFileInfoArgs {
                file: types::FileLocator::Path(path),
                target: self.id,
                with_download_info: download,
            })
            .await
    }

    pub async fn make_directory<S: MahSession + ?Sized>(
        &self,
        session: &S,
        path: Option<&str>,
        name: &str,
    ) -> Result<FileDetails, S::Error> {
        session
            .mk_dir(&types::MkDirArgs {
                directory: path.map_or_else(types::FileLocator::root, types::FileLocator::Path),
                target: self.id,
                directory_name: name,
            })
            .await
    }

    pub async fn upload_file<S: MahSession + ?Sized>(
        &self,
        session: &S,
        path: Option<Cow<'static, str>>,
        name: Cow<'static, str>,
        file: Bytes,
    ) -> Result<FileDetails, S::Error> {
        session
            .upload_file(self.id, path.unwrap_or(Cow::Borrowed("")), name, file)
            .await
    }

    pub async fn delete_file<S: MahSession + ?Sized>(
        &self,
        session: &S,
        path: &str,
    ) -> Result<(), S::Error> {
        session
            .delete_file(&types::FileArgs {
                file: types::FileLocator::Path(path),
                target: self.id,
            })
            .await
    }

    pub async fn move_file<S: MahSession + ?Sized>(
        &self,
        session: &S,
        path: &str,
        new_parent: &FileHandle,
    ) -> Result<(), S::Error> {
        session
            .move_file(&types::MoveFileArgs {
                file: types::FileLocator::Path(path),
                target: self.id,
                move_to: types::FileLocator::Id(&new_parent.id),
            })
            .await
    }

    pub async fn move_file_to_path<S: MahSession + ?Sized>(
        &self,
        session: &S,
        path: &str,
        new_parent_path: &str,
    ) -> Result<(), S::Error> {
        session
            .move_file(&types::MoveFileArgs {
                file: types::FileLocator::Path(path),
                target: self.id,
                move_to: types::FileLocator::Path(new_parent_path),
            })
            .await
    }

    pub async fn rename_file<S: MahSession + ?Sized>(
        &self,
        session: &S,
        path: &str,
        new_name: &str,
    ) -> Result<(), S::Error> {
        session
            .rename_file(&types::RenameFileArgs {
                file: types::FileLocator::Path(path),
                target: self.id,
                rename_to: new_name,
            })
            .await
    }

    pub async fn list_announcements<S: MahSession + ?Sized>(
        &self,
        session: &S,
        range: (i32, Option<i32>),
    ) -> Result<Vec<AnnouncementDetails>, S::Error> {
        session
            .list_announcement(&types::ListAnnouncementArgs {
                id: self.id,
                offset: range.0,
                size: range.1,
            })
            .await
    }

    pub async fn publish_announcement<'a, S: MahSession + ?Sized>(
        &self,
        session: &S,
        announcement: &'a Announcement<'a>,
    ) -> Result<AnnouncementDetails, S::Error> {
        session
            .publish_announcement(&types::PublishAnnouncementArgs {
                target: self.id,
                announcement,
            })
            .await
    }
}

#[async_trait]
impl SendMessage for GroupHandle {
    async fn send_message<'a, S: MahSession + ?Sized>(
        &self,
        session: &S,
        message: &'a OutgoingMessageContents<'a>,
    ) -> Result<MessageHandle, S::Error> {
        self.send_message(session, message).await
    }

    async fn upload_image<S: MahSession + ?Sized>(
        &self,
        session: &S,
        image: FileUpload,
    ) -> Result<ImageInfo, S::Error> {
        self.upload_image(session, image).await
    }

    async fn upload_voice<S: MahSession + ?Sized>(
        &self,
        session: &S,
        voice: FileUpload,
    ) -> Result<VoiceInfo, S::Error> {
        self.upload_voice(session, voice).await
    }
}

#[async_trait]
impl SendNudge for GroupHandle {
    async fn send_nudge<S: MahSession + ?Sized>(
        &self,
        session: &S,
        target: UserHandle,
    ) -> Result<(), S::Error> {
        self.send_nudge(session, target).await
    }
}

#[async_trait]
impl GetRoamingMessages for GroupHandle {
    async fn get_roaming_messages<S: MahSession + ?Sized>(
        &self,
        session: &S,
        start_time_secs: i64,
        end_time_secs: i64,
    ) -> Result<Vec<Message>, S::Error> {
        self.get_roaming_messages(session, start_time_secs, end_time_secs)
            .await
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct GroupDetails {
    pub id: i64,
    pub name: String,
    pub permission: MemberPermission,
}

impl GroupDetails {
    pub fn handle(&self) -> GroupHandle {
        GroupHandle { id: self.id }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MemberHandle {
    id: i64,
    group: GroupHandle,
}

impl MemberHandle {
    pub fn group(&self) -> GroupHandle {
        self.group
    }

    pub async fn resolve<S: MahSession + ?Sized>(
        &self,
        session: &S,
    ) -> Result<MemberInfo, S::Error> {
        session
            .get_member_info(&types::MemberArgs {
                target: self.group.id,
                member_id: self.id,
            })
            .await
    }

    pub async fn send_message<'a, S: MahSession + ?Sized>(
        &self,
        session: &S,
        message: &'a OutgoingMessageContents<'a>,
    ) -> Result<MessageHandle, S::Error> {
        Ok(MessageHandle {
            id: session
                .send_temp_message(&types::SendTempMessageArgs {
                    qq: self.id,
                    group: self.group.id,
                    contents: message,
                })
                .await?,
            context: self.id,
        })
    }

    pub async fn upload_image<S: MahSession + ?Sized>(
        &self,
        session: &S,
        image: FileUpload,
    ) -> Result<ImageInfo, S::Error> {
        session.upload_image(types::MediaType::Temp, image).await
    }

    pub async fn upload_voice<S: MahSession + ?Sized>(
        &self,
        session: &S,
        voice: FileUpload,
    ) -> Result<VoiceInfo, S::Error> {
        session.upload_voice(types::MediaType::Temp, voice).await
    }

    pub async fn get_profile<S: MahSession + ?Sized>(
        &self,
        session: &S,
    ) -> Result<Profile, S::Error> {
        self.to_user().get_profile(session).await
    }

    pub async fn mute<S: MahSession + ?Sized>(
        &self,
        session: &S,
        duration_secs: i32,
    ) -> Result<(), S::Error> {
        session
            .mute(&types::MuteArgs {
                target: self.group.id,
                member_id: self.id,
                time: duration_secs,
            })
            .await
    }

    pub async fn unmute<S: MahSession + ?Sized>(&self, session: &S) -> Result<(), S::Error> {
        session
            .unmute(&types::MemberArgs {
                target: self.group.id,
                member_id: self.id,
            })
            .await
    }

    pub async fn kick<S: MahSession + ?Sized>(
        &self,
        session: &S,
        message: Option<&str>,
        block: bool,
    ) -> Result<(), S::Error> {
        session
            .kick(&types::KickArgs {
                target: self.group.id,
                member_id: self.id,
                block,
                msg: message.unwrap_or_default(),
            })
            .await
    }

    pub async fn update_member_info<'a, S: MahSession + ?Sized>(
        &self,
        session: &S,
        info: &'a MemberInfoUpdate<'a>,
    ) -> Result<(), S::Error> {
        session
            .update_member_info(&types::UpdateMemberInfoArgs {
                target: self.group.id,
                member_id: self.id,
                info,
            })
            .await
    }

    pub async fn set_admin<S: MahSession + ?Sized>(
        &self,
        session: &S,
        is_admin: bool,
    ) -> Result<(), S::Error> {
        session
            .modify_member_admin(&types::ModifyMemberAdminArgs {
                target: self.group.id,
                member_id: self.id,
                assign: is_admin,
            })
            .await
    }
}

impl AnyUserHandle for MemberHandle {
    fn id(&self) -> i64 {
        self.id
    }
}

#[async_trait]
impl SendMessage for MemberHandle {
    async fn send_message<'a, S: MahSession + ?Sized>(
        &self,
        session: &S,
        message: &'a OutgoingMessageContents<'a>,
    ) -> Result<MessageHandle, S::Error> {
        self.send_message(session, message).await
    }

    async fn upload_image<S: MahSession + ?Sized>(
        &self,
        session: &S,
        image: FileUpload,
    ) -> Result<ImageInfo, S::Error> {
        self.upload_image(session, image).await
    }

    async fn upload_voice<S: MahSession + ?Sized>(
        &self,
        session: &S,
        voice: FileUpload,
    ) -> Result<VoiceInfo, S::Error> {
        self.upload_voice(session, voice).await
    }
}

#[async_trait]
impl GetProfile for MemberHandle {
    async fn get_profile<S: MahSession + ?Sized>(&self, session: &S) -> Result<Profile, S::Error> {
        self.get_profile(session).await
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberDetails {
    pub id: i64,
    pub member_name: String,
    pub special_title: String,
    pub permission: MemberPermission,
    #[serde(rename = "joinTimestamp")]
    pub join_time_secs: i32,
    #[serde(rename = "lastSpeakTimestamp")]
    pub last_speak_time_secs: i32,
    #[serde(rename = "muteTimeRemaining")]
    pub mute_time_remaining_secs: i32,
    pub group: GroupDetails,
}

impl MemberDetails {
    pub fn handle(&self) -> MemberHandle {
        self.group.handle().get_member(self.id)
    }

    pub fn join_time(&self) -> Option<SystemTime> {
        SystemTime::UNIX_EPOCH.checked_add(Duration::from_secs(self.join_time_secs as u64))
    }

    pub fn last_speak_time(&self) -> Option<SystemTime> {
        SystemTime::UNIX_EPOCH.checked_add(Duration::from_secs(self.last_speak_time_secs as u64))
    }

    pub fn mute_time_remaining(&self) -> Duration {
        Duration::from_secs(self.mute_time_remaining_secs as u64)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct MemberInfo {
    #[serde(flatten)]
    pub details: MemberDetails,
    #[serde(rename = "active")]
    pub activity: MemberActivity,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FileHandle {
    id: String,
    group: GroupHandle,
}

impl FileHandle {
    pub fn id(&self) -> &str {
        self.id.as_ref()
    }

    pub fn group(&self) -> GroupHandle {
        self.group
    }

    pub async fn resolve<S: MahSession + ?Sized>(
        &self,
        session: &S,
        download: bool,
    ) -> Result<FileDetails, S::Error> {
        session
            .get_file_info(&types::GetFileInfoArgs {
                file: types::FileLocator::Id(&self.id),
                target: self.group.id,
                with_download_info: download,
            })
            .await
    }

    pub async fn list<S: MahSession + ?Sized>(
        &self,
        session: &S,
        range: (i32, Option<i32>),
        download: bool,
    ) -> Result<Vec<FileDetails>, S::Error> {
        session
            .list_file(&types::ListFileArgs {
                directory: types::FileLocator::Id(&self.id),
                target: self.group.id,
                offset: range.0,
                size: range.1,
                with_download_info: download,
            })
            .await
    }

    pub async fn delete<S: MahSession + ?Sized>(&self, session: &S) -> Result<(), S::Error> {
        session
            .delete_file(&types::FileArgs {
                file: types::FileLocator::Id(&self.id),
                target: self.group.id,
            })
            .await
    }

    pub async fn move_<S: MahSession + ?Sized>(
        &self,
        session: &S,
        new_parent: &FileHandle,
    ) -> Result<(), S::Error> {
        session
            .move_file(&types::MoveFileArgs {
                file: types::FileLocator::Id(&self.id),
                target: self.group.id,
                move_to: types::FileLocator::Id(&new_parent.id),
            })
            .await
    }

    pub async fn move_to_path<S: MahSession + ?Sized>(
        &self,
        session: &S,
        new_parent_path: &str,
    ) -> Result<(), S::Error> {
        session
            .move_file(&types::MoveFileArgs {
                file: types::FileLocator::Id(&self.id),
                target: self.group.id,
                move_to: types::FileLocator::Path(new_parent_path),
            })
            .await
    }

    pub async fn rename<S: MahSession + ?Sized>(
        &self,
        session: &S,
        new_name: &str,
    ) -> Result<(), S::Error> {
        session
            .rename_file(&types::RenameFileArgs {
                file: types::FileLocator::Id(&self.id),
                target: self.group.id,
                rename_to: new_name,
            })
            .await
    }
}

fn deserialize_file_metadata<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<FileMetadata>, D::Error> {
    use serde::de::Error;

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Impl {
        is_file: bool,
        is_directory: bool,
        size: i64,
        sha1: Option<String>,
        md5: Option<String>,
        uploader_id: Option<i64>,
        upload_time: Option<i64>,
        last_modify_time: Option<i64>,
        download_info: Option<FileDownloadInfo>,
    }

    match Deserialize::deserialize(deserializer)? {
        Impl {
            is_file: true,
            is_directory: false,
            size,
            sha1: Some(sha1),
            md5: Some(md5),
            uploader_id: Some(uploader_id),
            upload_time: Some(upload_time_secs),
            last_modify_time: Some(last_modify_time_secs),
            download_info,
        } => Ok(Some(FileMetadata {
            size,
            sha1,
            md5,
            uploader_id,
            upload_time_secs,
            last_modify_time_secs,
            download_info,
        })),
        Impl {
            is_file: false,
            is_directory: true,
            size: 0,
            sha1: None,
            md5: None,
            uploader_id: None,
            upload_time: None,
            last_modify_time: None,
            download_info: None,
        } => Ok(None),
        _ => Err(D::Error::custom("expected a file or a directory")),
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileDetails {
    pub id: String,
    pub name: String,
    pub path: String,
    pub parent: Option<Box<FileDetails>>,
    #[serde(flatten, deserialize_with = "deserialize_file_metadata")]
    pub metadata: Option<FileMetadata>,
    #[serde(rename = "contact")]
    pub group: GroupDetails,
}

impl FileDetails {
    pub fn handle(&self) -> FileHandle {
        self.group.handle().get_file(self.id.clone())
    }

    pub fn into_handle(self) -> FileHandle {
        self.group.handle().get_file(self.id)
    }

    pub fn uploader(&self) -> Option<MemberHandle> {
        Some(self.metadata.as_ref()?.uploader(self.group.handle()))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AnnouncementHandle {
    id: String,
    group: GroupHandle,
}

impl AnnouncementHandle {
    pub fn id(&self) -> &str {
        self.id.as_ref()
    }

    pub fn group(&self) -> GroupHandle {
        self.group
    }

    pub async fn delete<S: MahSession + ?Sized>(&self, session: &S) -> Result<(), S::Error> {
        session
            .delete_announcement(&types::AnnouncementArgs {
                id: self.group.id,
                fid: &self.id,
            })
            .await
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnouncementDetails {
    #[serde(rename = "fid")]
    pub id: String,
    #[serde(rename = "content")]
    pub contents: String,
    #[serde(rename = "senderId")]
    pub publisher_id: i64,
    #[serde(rename = "publicationTime")]
    pub publication_time_secs: i64,
    #[serde(rename = "confirmedMembersCount")]
    pub confirmed_count: i32,
    pub all_confirmed: bool,
    pub group: GroupDetails,
}

impl AnnouncementDetails {
    pub fn handle(&self) -> AnnouncementHandle {
        self.group.handle().get_announcement(self.id.clone())
    }

    pub fn publication_time(&self) -> Option<SystemTime> {
        SystemTime::UNIX_EPOCH.checked_add(Duration::from_secs(self.publication_time_secs as u64))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct OtherClientHandle {
    id: i64,
}

impl OtherClientHandle {
    pub fn id(&self) -> i64 {
        self.id
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct OtherClientDetails {
    pub id: i64,
    pub platform: String,
}

impl OtherClientDetails {
    pub fn handle(&self) -> OtherClientHandle {
        OtherClientHandle { id: self.id }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MessageHandle {
    id: i32,
    context: i64,
}

impl MessageHandle {
    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn context(&self) -> i64 {
        self.context
    }

    pub async fn resolve<S: MahSession + ?Sized>(&self, session: &S) -> Result<Message, S::Error> {
        session
            .get_message_from_id(&types::MessageIdArgs {
                target: self.context,
                message_id: self.id,
            })
            .await
    }

    pub async fn recall<S: MahSession + ?Sized>(&self, session: &S) -> Result<(), S::Error> {
        session
            .recall(&types::MessageIdArgs {
                target: self.context,
                message_id: self.id,
            })
            .await
    }

    pub async fn set_essence<S: MahSession + ?Sized>(&self, session: &S) -> Result<(), S::Error> {
        session
            .set_essence(&types::MessageIdArgs {
                target: self.context,
                message_id: self.id,
            })
            .await
    }
}

#[doc(hidden)]
pub mod __ {
    pub use std::convert::Into;
}
