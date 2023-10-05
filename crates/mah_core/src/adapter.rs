use std::borrow::Cow;
use std::num::NonZeroU16;

use async_trait::async_trait;
pub use bytes::Bytes;
use serde::Deserialize;
use thiserror::Error;

use crate::message::{FriendMessage, Message};
use crate::{
    types, AnnouncementDetails, Command, FileDetails, FileUpload, FriendDetails, GroupConfig,
    GroupDetails, ImageInfo, MemberDetails, MemberInfo, Profile, VoiceInfo,
};

#[async_trait]
pub trait Mah: Sync {
    type Error: std::error::Error + Send + Sync + 'static;

    // region: about
    async fn about(&self) -> Result<types::AboutResult, Self::Error>;
    async fn get_bots_list(&self) -> Result<Vec<i64>, Self::Error>;
    // endregion
}

#[async_trait]
pub trait MahSession: Sync {
    type Error: std::error::Error + Send + Sync + 'static;

    // region: message
    async fn get_message_from_id(
        &self,
        args: &types::MessageIdArgs,
    ) -> Result<Message, Self::Error>;
    async fn send_friend_message(&self, args: &types::SendMessageArgs) -> Result<i32, Self::Error>;
    async fn send_group_message(&self, args: &types::SendMessageArgs) -> Result<i32, Self::Error>;
    async fn send_temp_message(
        &self,
        args: &types::SendTempMessageArgs,
    ) -> Result<i32, Self::Error>;
    async fn send_other_client_message(
        &self,
        args: &types::SendMessageArgs,
    ) -> Result<i32, Self::Error>;
    async fn upload_image(
        &self,
        media_type: types::MediaType,
        image: FileUpload,
    ) -> Result<ImageInfo, Self::Error>;
    async fn upload_voice(
        &self,
        media_type: types::MediaType,
        voice: FileUpload,
    ) -> Result<VoiceInfo, Self::Error>;
    async fn recall(&self, args: &types::MessageIdArgs) -> Result<(), Self::Error>;
    async fn nudge(&self, args: &types::NudgeArgs) -> Result<(), Self::Error>;
    async fn roaming_messages(
        &self,
        args: &types::RoamingMessagesArgs,
    ) -> Result<Vec<FriendMessage>, Self::Error>;
    // endregion

    // region: event
    async fn handle_new_friend_request(
        &self,
        args: &types::HandleNewFriendRequestArgs,
    ) -> Result<(), Self::Error>;
    async fn handle_member_join_request(
        &self,
        args: &types::HandleMemberJoinRequestArgs,
    ) -> Result<(), Self::Error>;
    async fn handle_bot_invited_join_group_request(
        &self,
        args: &types::HandleBotInvitedJoinGroupRequestArgs,
    ) -> Result<(), Self::Error>;
    // endregion

    // region: info
    async fn get_friend_list(&self) -> Result<Vec<FriendDetails>, Self::Error>;
    async fn get_group_list(&self) -> Result<Vec<GroupDetails>, Self::Error>;
    async fn get_member_list(
        &self,
        args: &types::TargetArgs,
    ) -> Result<Vec<MemberDetails>, Self::Error>;
    async fn latest_member_list(
        &self,
        args: &types::MultiMemberArgs,
    ) -> Result<Vec<MemberDetails>, Self::Error>;
    async fn get_bot_profile(&self) -> Result<Profile, Self::Error>;
    async fn get_friend_profile(&self, args: &types::TargetArgs) -> Result<Profile, Self::Error>;
    async fn get_member_profile(&self, args: &types::MemberArgs) -> Result<Profile, Self::Error>;
    async fn get_user_profile(&self, args: &types::TargetArgs) -> Result<Profile, Self::Error>;
    // endregion

    // region: friend
    async fn delete_friend(&self, args: &types::TargetArgs) -> Result<(), Self::Error>;
    // endregion

    // region: group
    async fn mute_all(&self, args: &types::TargetArgs) -> Result<(), Self::Error>;
    async fn unmute_all(&self, args: &types::TargetArgs) -> Result<(), Self::Error>;
    async fn mute(&self, args: &types::MuteArgs) -> Result<(), Self::Error>;
    async fn unmute(&self, args: &types::MemberArgs) -> Result<(), Self::Error>;
    async fn kick(&self, args: &types::KickArgs) -> Result<(), Self::Error>;
    async fn quit(&self, args: &types::TargetArgs) -> Result<(), Self::Error>;
    async fn set_essence(&self, args: &types::MessageIdArgs) -> Result<(), Self::Error>;
    async fn get_group_config(&self, args: &types::TargetArgs) -> Result<GroupConfig, Self::Error>;
    async fn update_group_config(
        &self,
        args: &types::UpdateGroupConfigArgs,
    ) -> Result<(), Self::Error>;
    async fn get_member_info(&self, args: &types::MemberArgs) -> Result<MemberInfo, Self::Error>;
    async fn update_member_info(
        &self,
        args: &types::UpdateMemberInfoArgs,
    ) -> Result<(), Self::Error>;
    async fn modify_member_admin(
        &self,
        args: &types::ModifyMemberAdminArgs,
    ) -> Result<(), Self::Error>;
    // endregion

    // region: about
    async fn get_session_info(&self) -> Result<types::GetSessionInfoResult, Self::Error>;
    // endregion

    // region: file
    async fn list_file(&self, args: &types::ListFileArgs) -> Result<Vec<FileDetails>, Self::Error>;
    async fn get_file_info(
        &self,
        args: &types::GetFileInfoArgs,
    ) -> Result<FileDetails, Self::Error>;
    async fn mk_dir(&self, args: &types::MkDirArgs) -> Result<FileDetails, Self::Error>;
    async fn upload_file(
        &self,
        group: i64,
        path: Cow<'static, str>,
        name: Cow<'static, str>,
        file: Bytes,
    ) -> Result<FileDetails, Self::Error>;
    async fn delete_file(&self, args: &types::FileArgs) -> Result<(), Self::Error>;
    async fn move_file(&self, args: &types::MoveFileArgs) -> Result<(), Self::Error>;
    async fn rename_file(&self, args: &types::RenameFileArgs) -> Result<(), Self::Error>;
    // endregion

    // region: command
    async fn execute_command(&self, args: &types::ExecuteCommandArgs) -> Result<(), Self::Error>;
    async fn register_command(&self, args: &Command) -> Result<(), Self::Error>;
    // endregion

    // region: announcement
    async fn list_announcement(
        &self,
        args: &types::ListAnnouncementArgs,
    ) -> Result<Vec<AnnouncementDetails>, Self::Error>;
    async fn publish_announcement(
        &self,
        args: &types::PublishAnnouncementArgs,
    ) -> Result<AnnouncementDetails, Self::Error>;
    async fn delete_announcement(&self, args: &types::AnnouncementArgs) -> Result<(), Self::Error>;
    // endregion
}

#[derive(Clone, Debug, Deserialize, Error)]
#[error("{message}")]
pub struct Error {
    pub code: NonZeroU16,
    #[serde(default, rename = "msg")]
    pub message: String,
}
