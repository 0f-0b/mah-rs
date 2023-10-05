#![forbid(unsafe_code)]

pub mod fetch;

use std::borrow::Cow;
use std::fmt::Debug;
use std::num::NonZeroU32;

use async_trait::async_trait;
use mah_core::adapter::{self, Bytes, Mah, MahSession};
use mah_core::event::MessageOrEvent;
use mah_core::message::{FriendMessage, Message};
use mah_core::{
    types, AnnouncementDetails, Command, FileDetails, FileUpload, FriendDetails, GroupConfig,
    GroupDetails, ImageInfo, MemberDetails, MemberInfo, Profile, VoiceInfo,
};
use once_cell::sync::Lazy;
use reqwest::header::HeaderValue;
pub use reqwest::Url;
use reqwest::{multipart, Method, Request, RequestBuilder, Response};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use thiserror::Error;
use tokio::sync::mpsc;
pub use tokio::time::Duration;

use self::fetch::{DefaultFetch, Fetch};

#[async_trait]
trait HttpAdapterHandler {
    fn base_url(&self) -> &Url;

    fn get(&self, path: &str) -> RequestBuilder {
        self.request(Method::GET, path)
    }

    fn post(&self, path: &str) -> RequestBuilder {
        self.request(Method::POST, path)
    }

    fn request(&self, method: Method, path: &str) -> RequestBuilder {
        static REQUEST_BUILDER: Lazy<reqwest::Client> = Lazy::new(Default::default);
        REQUEST_BUILDER.request(method, self.base_url().join(path).unwrap())
    }

    async fn fetch(&self, request: Request) -> Result<Response, reqwest::Error>;

    async fn validate<T: DeserializeOwned>(&self, request: Request) -> Result<T, HttpAdapterError> {
        let value = self
            .fetch(request)
            .await?
            .json::<serde_json::Value>()
            .await?;
        if let Ok(err) = adapter::Error::deserialize(&value) {
            return Err(err.into());
        }
        Ok(T::deserialize(value)?)
    }

    async fn data<T: DeserializeOwned>(&self, request: Request) -> Result<T, HttpAdapterError> {
        #[derive(Debug, Deserialize)]
        struct Data<T> {
            data: T,
        }

        self.validate(request).await.map(|Data { data }| data)
    }

    async fn send(&self, request: Request) -> Result<i32, HttpAdapterError> {
        types::SendMessageResult::into(self.validate(request).await?)
    }
}

#[derive(Clone, Debug)]
pub struct HttpAdapter<F = DefaultFetch> {
    verify_key: String,
    base_url: Url,
    fetch: F,
}

impl HttpAdapter<DefaultFetch> {
    pub fn new(endpoint: Url, verify_key: Option<String>) -> Self {
        Self::with_fetch(endpoint, verify_key, DefaultFetch::new())
    }
}

impl<F: Fetch> HttpAdapter<F> {
    pub fn with_fetch(endpoint: Url, verify_key: Option<String>, fetch: F) -> Self {
        assert!(endpoint.scheme() == "http" || endpoint.scheme() == "https");
        let mut base_url = endpoint;
        base_url
            .path_segments_mut()
            .unwrap()
            .pop_if_empty()
            .push("");
        Self {
            verify_key: verify_key.unwrap_or_default(),
            base_url,
            fetch,
        }
    }

    // region: verify
    pub async fn verify(&self) -> Result<HttpAdapterSession<F>, HttpAdapterError> {
        self.validate(
            self.post("verify")
                .json(&types::VerifyArgs {
                    verify_key: &self.verify_key,
                })
                .build()?,
        )
        .await
        .map(|types::VerifyResult { session }| HttpAdapterSession {
            session_key: {
                let mut value = HeaderValue::from_str(&session).unwrap();
                value.set_sensitive(true);
                value
            },
            fetch: self.fetch.clone(),
            base_url: self.base_url.clone(),
        })
    }
    // endregion
}

#[async_trait]
impl<F: Fetch> Mah for HttpAdapter<F> {
    type Error = HttpAdapterError;

    // region: about
    async fn about(&self) -> Result<types::AboutResult, Self::Error> {
        self.data(self.get("about").build()?).await
    }

    async fn get_bots_list(&self) -> Result<Vec<i64>, Self::Error> {
        self.data(self.get("botList").build()?).await
    }
    // endregion
}

#[async_trait]
impl<F: Fetch> HttpAdapterHandler for HttpAdapter<F> {
    fn base_url(&self) -> &Url {
        &self.base_url
    }

    async fn fetch(&self, request: Request) -> Result<Response, reqwest::Error> {
        self.fetch.fetch(request).await
    }
}

#[derive(Clone, Debug)]
pub struct HttpAdapterSession<F = DefaultFetch> {
    base_url: Url,
    session_key: HeaderValue,
    fetch: F,
}

impl<F: Fetch> HttpAdapterSession<F> {
    // region: verify
    pub async fn bind(&self, args: &types::BindArgs) -> Result<(), HttpAdapterError> {
        self.validate(self.post("bind").json(args).build()?).await
    }

    pub async fn release(&self, args: &types::BindArgs) -> Result<(), HttpAdapterError> {
        self.validate(self.post("release").json(args).build()?)
            .await
    }
    // endregion

    // region: message
    pub async fn count_message(&self) -> Result<i32, HttpAdapterError> {
        self.data(self.get("countMessage").build()?).await
    }

    pub async fn fetch_message(
        &self,
        args: &types::CountArgs,
    ) -> Result<Vec<MessageOrEvent>, HttpAdapterError> {
        self.data(self.get("fetchMessage").query(args).build()?)
            .await
    }

    pub async fn fetch_latest_message(
        &self,
        args: &types::CountArgs,
    ) -> Result<Vec<MessageOrEvent>, HttpAdapterError> {
        self.data(self.get("fetchLatestMessage").query(args).build()?)
            .await
    }

    pub async fn peek_message(
        &self,
        args: &types::CountArgs,
    ) -> Result<Vec<MessageOrEvent>, HttpAdapterError> {
        self.data(self.get("peekMessage").query(args).build()?)
            .await
    }

    pub async fn peek_latest_message(
        &self,
        args: &types::CountArgs,
    ) -> Result<Vec<MessageOrEvent>, HttpAdapterError> {
        self.data(self.get("peekLatestMessage").query(args).build()?)
            .await
    }
    // endregion
}

#[async_trait]
impl<F: Fetch> MahSession for HttpAdapterSession<F> {
    type Error = HttpAdapterError;

    // region: message
    async fn get_message_from_id(
        &self,
        args: &types::MessageIdArgs,
    ) -> Result<Message, Self::Error> {
        self.data(self.get("messageFromId").query(args).build()?)
            .await
    }

    async fn send_friend_message(&self, args: &types::SendMessageArgs) -> Result<i32, Self::Error> {
        self.send(self.post("sendFriendMessage").json(args).build()?)
            .await
    }

    async fn send_group_message(&self, args: &types::SendMessageArgs) -> Result<i32, Self::Error> {
        self.send(self.post("sendGroupMessage").json(args).build()?)
            .await
    }

    async fn send_temp_message(
        &self,
        args: &types::SendTempMessageArgs,
    ) -> Result<i32, Self::Error> {
        self.send(self.post("sendTempMessage").json(args).build()?)
            .await
    }

    async fn send_other_client_message(
        &self,
        args: &types::SendMessageArgs,
    ) -> Result<i32, Self::Error> {
        self.send(self.post("sendOtherClientMessage").json(args).build()?)
            .await
    }

    async fn upload_image(
        &self,
        media_type: types::MediaType,
        image: FileUpload,
    ) -> Result<ImageInfo, Self::Error> {
        let form = multipart::Form::new().text("type", <&'static str>::from(media_type));
        let form = match image {
            FileUpload::Url(url) => form.text("url", url),
            FileUpload::Bytes(bytes) => form.part("img", multipart::Part::stream(bytes)),
        };
        self.validate(self.post("uploadImage").multipart(form).build()?)
            .await
    }

    async fn upload_voice(
        &self,
        media_type: types::MediaType,
        voice: FileUpload,
    ) -> Result<VoiceInfo, Self::Error> {
        let form = multipart::Form::new().text("type", <&'static str>::from(media_type));
        let form = match voice {
            FileUpload::Url(url) => form.text("url", url),
            FileUpload::Bytes(bytes) => form.part("voice", multipart::Part::stream(bytes)),
        };
        self.validate(self.post("uploadVoice").multipart(form).build()?)
            .await
    }

    async fn recall(&self, args: &types::MessageIdArgs) -> Result<(), Self::Error> {
        self.validate(self.post("recall").json(args).build()?).await
    }

    async fn nudge(&self, args: &types::NudgeArgs) -> Result<(), Self::Error> {
        self.validate(self.post("sendNudge").json(args).build()?)
            .await
    }

    async fn roaming_messages(
        &self,
        args: &types::RoamingMessagesArgs,
    ) -> Result<Vec<FriendMessage>, Self::Error> {
        self.data(self.post("roamingMessages").json(args).build()?)
            .await
    }
    // endregion

    // region: event
    async fn handle_new_friend_request(
        &self,
        args: &types::HandleNewFriendRequestArgs,
    ) -> Result<(), Self::Error> {
        self.validate(self.post("resp/newFriendRequestEvent").json(args).build()?)
            .await
    }

    async fn handle_member_join_request(
        &self,
        args: &types::HandleMemberJoinRequestArgs,
    ) -> Result<(), Self::Error> {
        self.validate(
            self.post("resp/memberJoinRequestEvent")
                .json(args)
                .build()?,
        )
        .await
    }

    async fn handle_bot_invited_join_group_request(
        &self,
        args: &types::HandleBotInvitedJoinGroupRequestArgs,
    ) -> Result<(), Self::Error> {
        self.validate(
            self.post("resp/botInvitedJoinGroupRequestEvent")
                .json(args)
                .build()?,
        )
        .await
    }
    // endregion

    // region: info
    async fn get_friend_list(&self) -> Result<Vec<FriendDetails>, Self::Error> {
        self.data(self.get("friendList").build()?).await
    }

    async fn get_group_list(&self) -> Result<Vec<GroupDetails>, Self::Error> {
        self.data(self.get("groupList").build()?).await
    }

    async fn get_member_list(
        &self,
        args: &types::TargetArgs,
    ) -> Result<Vec<MemberDetails>, Self::Error> {
        self.data(self.get("memberList").query(args).build()?).await
    }

    async fn latest_member_list(
        &self,
        args: &types::MultiMemberArgs,
    ) -> Result<Vec<MemberDetails>, Self::Error> {
        self.data(self.get("latestMemberList").query(args).build()?)
            .await
    }

    async fn get_bot_profile(&self) -> Result<Profile, Self::Error> {
        self.validate(self.get("botProfile").build()?).await
    }

    async fn get_friend_profile(&self, args: &types::TargetArgs) -> Result<Profile, Self::Error> {
        self.validate(self.get("friendProfile").query(args).build()?)
            .await
    }

    async fn get_member_profile(&self, args: &types::MemberArgs) -> Result<Profile, Self::Error> {
        self.validate(self.get("memberProfile").query(args).build()?)
            .await
    }

    async fn get_user_profile(&self, args: &types::TargetArgs) -> Result<Profile, Self::Error> {
        self.validate(self.get("userProfile").query(args).build()?)
            .await
    }
    // endregion

    // region: friend
    async fn delete_friend(&self, args: &types::TargetArgs) -> Result<(), Self::Error> {
        self.validate(self.post("deleteFriend").json(args).build()?)
            .await
    }
    // endregion

    // region: group
    async fn mute_all(&self, args: &types::TargetArgs) -> Result<(), Self::Error> {
        self.validate(self.post("muteAll").json(args).build()?)
            .await
    }

    async fn unmute_all(&self, args: &types::TargetArgs) -> Result<(), Self::Error> {
        self.validate(self.post("unmuteAll").json(args).build()?)
            .await
    }

    async fn mute(&self, args: &types::MuteArgs) -> Result<(), Self::Error> {
        self.validate(self.post("mute").json(args).build()?).await
    }

    async fn unmute(&self, args: &types::MemberArgs) -> Result<(), Self::Error> {
        self.validate(self.post("unmute").json(args).build()?).await
    }

    async fn kick(&self, args: &types::KickArgs) -> Result<(), Self::Error> {
        self.validate(self.post("kick").json(args).build()?).await
    }

    async fn quit(&self, args: &types::TargetArgs) -> Result<(), Self::Error> {
        self.validate(self.post("quit").json(args).build()?).await
    }

    async fn set_essence(&self, args: &types::MessageIdArgs) -> Result<(), Self::Error> {
        self.validate(self.post("setEssence").json(args).build()?)
            .await
    }

    async fn get_group_config(&self, args: &types::TargetArgs) -> Result<GroupConfig, Self::Error> {
        self.validate(self.get("groupConfig").query(args).build()?)
            .await
    }

    async fn update_group_config(
        &self,
        args: &types::UpdateGroupConfigArgs,
    ) -> Result<(), Self::Error> {
        self.validate(self.post("groupConfig").json(args).build()?)
            .await
    }

    async fn get_member_info(&self, args: &types::MemberArgs) -> Result<MemberInfo, Self::Error> {
        self.validate(self.get("memberInfo").query(args).build()?)
            .await
    }

    async fn update_member_info(
        &self,
        args: &types::UpdateMemberInfoArgs,
    ) -> Result<(), Self::Error> {
        self.validate(self.post("memberInfo").json(args).build()?)
            .await
    }

    async fn modify_member_admin(
        &self,
        args: &types::ModifyMemberAdminArgs,
    ) -> Result<(), Self::Error> {
        self.validate(self.post("memberAdmin").json(args).build()?)
            .await
    }
    // endregion

    // region: about
    async fn get_session_info(&self) -> Result<types::GetSessionInfoResult, Self::Error> {
        self.data(self.get("sessionInfo").build()?).await
    }
    // endregion

    // region: file
    async fn list_file(&self, args: &types::ListFileArgs) -> Result<Vec<FileDetails>, Self::Error> {
        self.data(self.get("file/list").query(args).build()?).await
    }

    async fn get_file_info(
        &self,
        args: &types::GetFileInfoArgs,
    ) -> Result<FileDetails, Self::Error> {
        self.data(self.get("file/info").query(args).build()?).await
    }

    async fn mk_dir(&self, args: &types::MkDirArgs) -> Result<FileDetails, Self::Error> {
        self.data(self.post("file/mkdir").json(args).build()?).await
    }

    async fn upload_file(
        &self,
        group: i64,
        path: Cow<'static, str>,
        name: Cow<'static, str>,
        file: Bytes,
    ) -> Result<FileDetails, Self::Error> {
        self.data(
            self.post("file/upload")
                .multipart(
                    multipart::Form::new()
                        .text("path", path)
                        .text("type", "group")
                        .text("target", group.to_string())
                        .part("file", multipart::Part::stream(file).file_name(name)),
                )
                .build()?,
        )
        .await
    }

    async fn delete_file(&self, args: &types::FileArgs) -> Result<(), Self::Error> {
        self.validate(self.post("file/delete").json(args).build()?)
            .await
    }

    async fn move_file(&self, args: &types::MoveFileArgs) -> Result<(), Self::Error> {
        self.validate(self.post("file/move").json(args).build()?)
            .await
    }

    async fn rename_file(&self, args: &types::RenameFileArgs) -> Result<(), Self::Error> {
        self.validate(self.post("file/rename").json(args).build()?)
            .await
    }
    // endregion

    // region: command
    async fn execute_command(&self, args: &types::ExecuteCommandArgs) -> Result<(), Self::Error> {
        self.validate(self.post("cmd/execute").json(args).build()?)
            .await
    }

    async fn register_command(&self, args: &Command) -> Result<(), Self::Error> {
        self.validate(self.post("cmd/register").json(args).build()?)
            .await
    }
    // endregion

    // region: announcement
    async fn list_announcement(
        &self,
        args: &types::ListAnnouncementArgs,
    ) -> Result<Vec<AnnouncementDetails>, Self::Error> {
        self.data(self.get("anno/list").query(args).build()?).await
    }

    async fn publish_announcement(
        &self,
        args: &types::PublishAnnouncementArgs,
    ) -> Result<AnnouncementDetails, Self::Error> {
        self.data(self.post("anno/publish").json(args).build()?)
            .await
    }

    async fn delete_announcement(&self, args: &types::AnnouncementArgs) -> Result<(), Self::Error> {
        self.validate(self.post("anno/delete").json(args).build()?)
            .await
    }
    // endregion
}

#[async_trait]
impl<F: Fetch> HttpAdapterHandler for HttpAdapterSession<F> {
    fn base_url(&self) -> &Url {
        &self.base_url
    }

    async fn fetch(&self, mut request: Request) -> Result<Response, reqwest::Error> {
        request
            .headers_mut()
            .insert("sessionkey", self.session_key.clone());
        self.fetch.fetch(request).await
    }
}

#[derive(Clone, Copy, Debug)]
pub struct HttpAdapterEvents {
    buffer: usize,
    batch_size: Option<NonZeroU32>,
    poll_interval: Duration,
}

impl HttpAdapterEvents {
    pub fn new() -> Self {
        Self {
            buffer: 1,
            batch_size: None,
            poll_interval: Duration::from_millis(50),
        }
    }

    pub fn buffer(self, buffer: usize) -> Self {
        Self { buffer, ..self }
    }

    pub fn batch_size(self, batch_size: Option<NonZeroU32>) -> Self {
        if let Some(batch_size) = batch_size {
            assert!(batch_size.get() <= i32::MAX as u32);
        }
        Self { batch_size, ..self }
    }

    pub fn poll_interval(self, poll_interval: Duration) -> Self {
        Self {
            poll_interval,
            ..self
        }
    }

    pub fn listen<F: Fetch>(
        self,
        session: impl AsRef<HttpAdapterSession<F>> + Send + 'static,
        mut on_error: impl FnMut(HttpAdapterError) + Send + 'static,
    ) -> mpsc::Receiver<MessageOrEvent> {
        let (tx, rx) = mpsc::channel(self.buffer);
        let args = types::CountArgs {
            count: self.batch_size,
        };
        let poll_interval = self.poll_interval;
        tokio::spawn(async move {
            let session = session.as_ref();
            loop {
                let events = loop {
                    match session.fetch_message(&args).await {
                        Ok(events) => {
                            if !events.is_empty() {
                                break events;
                            }
                        }
                        Err(err) => {
                            on_error(err);
                        }
                    }
                    if tokio::time::timeout(poll_interval, tx.closed())
                        .await
                        .is_ok()
                    {
                        return;
                    }
                };
                for event in events {
                    let _ = tx.send(event).await;
                }
            }
        });
        rx
    }
}

impl Default for HttpAdapterEvents {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Error)]
pub enum HttpAdapterError {
    #[error("failed to fetch: {0}")]
    Fetch(#[from] reqwest::Error),
    #[error("invalid json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("mirai error: {0}")]
    Mirai(#[from] adapter::Error),
}
