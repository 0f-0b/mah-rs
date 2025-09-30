use std::time::{Duration, SystemTime};

use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Deserializer};

use crate::adapter::MahSession;
use crate::message::{
    FriendMessage, FriendSyncMessage, GroupMessage, GroupSyncMessage, IncomingMessageContents,
    IncomingMessageNode, Message, OtherClientMessage, StrangerMessage, StrangerSyncMessage,
    TempMessage, TempSyncMessage,
};
use crate::{
    Bot, FriendDetails, FriendHandle, GroupDetails, GroupHandle, GroupHonor, MemberDetails,
    MemberHandle, MemberPermission, MessageHandle, OtherClientDetails, StrangerDetails,
    StrangerHandle, UserHandle, types,
};

#[enum_dispatch]
#[allow(dead_code)]
trait AnyEvent {}

#[derive(Clone, Debug, Deserialize)]
pub struct BotOnlineEvent {
    #[serde(rename = "qq")]
    pub id: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct BotOfflineActiveEvent {
    #[serde(rename = "qq")]
    pub id: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct BotOfflineForcedEvent {
    #[serde(rename = "qq")]
    pub id: i64,
    pub title: String,
    pub message: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct BotOfflineDroppedEvent {
    #[serde(rename = "qq")]
    pub id: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct BotReloginEvent {
    #[serde(rename = "qq")]
    pub id: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct BotMuteEvent {
    #[serde(rename = "durationSeconds")]
    pub duration_secs: i32,
    pub operator: MemberDetails,
}

impl BotMuteEvent {
    pub fn duration(&self) -> Duration {
        Duration::from_secs(self.duration_secs as u64)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct BotUnmuteEvent {
    pub operator: MemberDetails,
}

#[derive(Clone, Debug, Deserialize)]
pub struct BotJoinGroupEvent {
    pub group: GroupDetails,
    #[serde(rename = "invitor")]
    pub inviter: Option<MemberDetails>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct BotLeaveGroupActiveEvent {
    pub group: GroupDetails,
}

#[derive(Clone, Debug, Deserialize)]
pub struct BotLeaveGroupKickedEvent {
    pub group: GroupDetails,
    pub operator: MemberDetails,
}

#[derive(Clone, Debug, Deserialize)]
pub struct BotLeaveGroupDisbandEvent {
    pub group: GroupDetails,
    pub operator: MemberDetails,
}

#[derive(Clone, Debug, Deserialize)]
pub struct BotPermissionChangeEvent {
    pub group: GroupDetails,
    #[serde(rename = "origin")]
    pub original: MemberPermission,
    pub current: MemberPermission,
}

#[derive(Clone, Debug)]
pub struct StrangerNudgeEvent {
    pub context: StrangerDetails,
    pub from_id: i64,
    pub to_id: i64,
    pub action: String,
    pub suffix: String,
}

impl StrangerNudgeEvent {
    pub fn from(&self) -> StrangerHandle {
        Bot.get_stranger(self.from_id)
    }

    pub fn to(&self) -> StrangerHandle {
        Bot.get_stranger(self.to_id)
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FriendMessageRecallEvent {
    pub message_id: i32,
    #[serde(rename = "authorId")]
    pub sender_id: i64,
    #[serde(rename = "time")]
    pub time_secs: i64,
}

impl FriendMessageRecallEvent {
    pub fn message(&self) -> Option<MessageHandle> {
        (self.message_id != 0).then_some(Bot.get_message(self.message_id, self.sender_id))
    }

    pub fn sender(&self) -> FriendHandle {
        Bot.get_friend(self.sender_id)
    }

    pub fn time(&self) -> Option<SystemTime> {
        SystemTime::UNIX_EPOCH.checked_add(Duration::from_secs(self.time_secs as u64))
    }
}

#[derive(Clone, Debug)]
pub struct FriendNudgeEvent {
    pub context: FriendDetails,
    pub from_id: i64,
    pub to_id: i64,
    pub action: String,
    pub suffix: String,
}

impl FriendNudgeEvent {
    pub fn from(&self) -> FriendHandle {
        Bot.get_friend(self.from_id)
    }

    pub fn to(&self) -> FriendHandle {
        Bot.get_friend(self.to_id)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct FriendAddEvent {
    pub friend: FriendDetails,
    #[serde(rename = "stranger")]
    pub was_stranger: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct FriendDeleteEvent {
    pub friend: FriendDetails,
}

#[derive(Clone, Debug, Deserialize)]
pub struct FriendNicknameChangeEvent {
    pub friend: FriendDetails,
    #[serde(rename = "from")]
    pub original: String,
    #[serde(rename = "to")]
    pub current: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct FriendTypingEvent {
    pub friend: FriendDetails,
    #[serde(rename = "inputting")]
    pub typing: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupMessageRecallEvent {
    pub message_id: i32,
    #[serde(rename = "group")]
    pub context: GroupDetails,
    #[serde(rename = "authorId")]
    pub sender_id: i64,
    #[serde(rename = "time")]
    pub time_secs: i64,
    pub operator: Option<MemberDetails>,
}

impl GroupMessageRecallEvent {
    pub fn message(&self) -> Option<MessageHandle> {
        (self.message_id != 0).then_some(Bot.get_message(self.message_id, self.context.id))
    }

    pub fn sender(&self) -> MemberHandle {
        self.context.handle().get_member(self.sender_id)
    }

    pub fn time(&self) -> Option<SystemTime> {
        SystemTime::UNIX_EPOCH.checked_add(Duration::from_secs(self.time_secs as u64))
    }

    pub fn is_operator(&self) -> bool {
        self.operator.is_none()
    }
}

#[derive(Clone, Debug)]
pub struct GroupNudgeEvent {
    pub context: GroupDetails,
    pub from_id: i64,
    pub to_id: i64,
    pub action: String,
    pub suffix: String,
}

impl GroupNudgeEvent {
    pub fn from(&self) -> MemberHandle {
        self.context.handle().get_member(self.from_id)
    }

    pub fn to(&self) -> MemberHandle {
        self.context.handle().get_member(self.to_id)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct GroupNameChangeEvent {
    pub group: GroupDetails,
    #[serde(rename = "origin")]
    pub original: String,
    pub current: String,
    pub operator: Option<MemberDetails>,
}

impl GroupNameChangeEvent {
    pub fn is_operator(&self) -> bool {
        self.operator.is_none()
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct GroupMuteAllEvent {
    pub group: GroupDetails,
    #[serde(rename = "origin")]
    pub original: bool,
    pub current: bool,
    pub operator: Option<MemberDetails>,
}

impl GroupMuteAllEvent {
    pub fn is_operator(&self) -> bool {
        self.operator.is_none()
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct GroupAllowAnonymousChatEvent {
    pub group: GroupDetails,
    #[serde(rename = "origin")]
    pub original: bool,
    pub current: bool,
    pub operator: Option<MemberDetails>,
}

impl GroupAllowAnonymousChatEvent {
    pub fn is_operator(&self) -> bool {
        self.operator.is_none()
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct GroupAllowConfessTalkEvent {
    pub group: GroupDetails,
    #[serde(rename = "origin")]
    pub original: bool,
    pub current: bool,
    #[serde(rename = "isByBot")]
    pub is_operator: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct GroupAllowMemberInviteEvent {
    pub group: GroupDetails,
    #[serde(rename = "origin")]
    pub original: bool,
    pub current: bool,
    pub operator: Option<MemberDetails>,
}

impl GroupAllowMemberInviteEvent {
    pub fn is_operator(&self) -> bool {
        self.operator.is_none()
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct MemberMuteEvent {
    pub member: MemberDetails,
    #[serde(rename = "durationSeconds")]
    pub duration_secs: i32,
    pub operator: Option<MemberDetails>,
}

impl MemberMuteEvent {
    pub fn duration(&self) -> Duration {
        Duration::from_secs(self.duration_secs as u64)
    }

    pub fn is_operator(&self) -> bool {
        self.operator.is_none()
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct MemberUnmuteEvent {
    pub member: MemberDetails,
    pub operator: Option<MemberDetails>,
}

impl MemberUnmuteEvent {
    pub fn is_operator(&self) -> bool {
        self.operator.is_none()
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct MemberJoinEvent {
    pub member: MemberDetails,
    #[serde(rename = "invitor")]
    pub inviter: Option<MemberDetails>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MemberLeaveActiveEvent {
    pub member: MemberDetails,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MemberLeaveKickedEvent {
    pub member: MemberDetails,
    pub operator: Option<MemberDetails>,
}

impl MemberLeaveKickedEvent {
    pub fn is_operator(&self) -> bool {
        self.operator.is_none()
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct MemberNameChangeEvent {
    pub member: MemberDetails,
    #[serde(rename = "origin")]
    pub original: String,
    pub current: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MemberSpecialTitleChangeEvent {
    pub member: MemberDetails,
    #[serde(rename = "origin")]
    pub original: String,
    pub current: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MemberPermissionChangeEvent {
    pub member: MemberDetails,
    #[serde(rename = "origin")]
    pub original: MemberPermission,
    pub current: MemberPermission,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MemberHonorChangeEvent {
    pub member: MemberDetails,
    pub action: MemberHonorChangeAction,
    pub honor: GroupHonor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemberHonorChangeAction {
    Achieve,
    Lose,
}

#[derive(Clone, Debug, Deserialize)]
pub struct OtherClientOnlineEvent {
    pub client: OtherClientDetails,
}

#[derive(Clone, Debug, Deserialize)]
pub struct OtherClientOfflineEvent {
    pub client: OtherClientDetails,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewFriendRequestEvent {
    pub event_id: i64,
    pub from_id: i64,
    #[serde(rename = "nick")]
    pub from_nickname: String,
    pub group_id: i64,
    pub message: String,
}

impl NewFriendRequestEvent {
    pub fn from(&self) -> UserHandle {
        Bot.get_user(self.from_id)
    }

    pub fn group(&self) -> Option<GroupHandle> {
        (self.group_id != 0).then_some(Bot.get_group(self.group_id))
    }

    pub async fn accept<S: MahSession + ?Sized>(&self, session: &S) -> Result<(), S::Error> {
        session
            .handle_new_friend_request(&types::HandleNewFriendRequestArgs {
                event_id: self.event_id,
                from_id: self.from_id,
                operation: types::NewFriendRequestOperation::Accept,
            })
            .await
    }

    pub async fn reject<S: MahSession + ?Sized>(
        &self,
        session: &S,
        block: bool,
    ) -> Result<(), S::Error> {
        session
            .handle_new_friend_request(&types::HandleNewFriendRequestArgs {
                event_id: self.event_id,
                from_id: self.from_id,
                operation: if block {
                    types::NewFriendRequestOperation::RejectAndBlock
                } else {
                    types::NewFriendRequestOperation::Reject
                },
            })
            .await
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberJoinRequestEvent {
    pub event_id: i64,
    pub from_id: i64,
    #[serde(rename = "nick")]
    pub from_nickname: String,
    pub group_id: i64,
    pub group_name: String,
    #[serde(rename = "invitorId")]
    pub inviter_id: Option<i64>,
    pub message: String,
}

impl MemberJoinRequestEvent {
    pub fn from(&self) -> UserHandle {
        Bot.get_user(self.from_id)
    }

    pub fn group(&self) -> GroupHandle {
        Bot.get_group(self.group_id)
    }

    pub fn inviter(&self) -> Option<MemberHandle> {
        Some(Bot.get_group(self.group_id).get_member(self.inviter_id?))
    }

    pub async fn accept<S: MahSession + ?Sized>(&self, session: &S) -> Result<(), S::Error> {
        session
            .handle_member_join_request(&types::HandleMemberJoinRequestArgs {
                event_id: self.event_id,
                from_id: self.from_id,
                group_id: self.group_id,
                operation: types::MemberJoinRequestOperation::Accept,
                message: "",
            })
            .await
    }

    pub async fn reject<S: MahSession + ?Sized>(
        &self,
        session: &S,
        message: Option<&str>,
        block: bool,
    ) -> Result<(), S::Error> {
        session
            .handle_member_join_request(&types::HandleMemberJoinRequestArgs {
                event_id: self.event_id,
                from_id: self.from_id,
                group_id: self.group_id,
                operation: if block {
                    types::MemberJoinRequestOperation::RejectAndBlock
                } else {
                    types::MemberJoinRequestOperation::Reject
                },
                message: message.unwrap_or_default(),
            })
            .await
    }

    pub async fn ignore<S: MahSession + ?Sized>(
        &self,
        session: &S,
        block: bool,
    ) -> Result<(), S::Error> {
        session
            .handle_member_join_request(&types::HandleMemberJoinRequestArgs {
                event_id: self.event_id,
                from_id: self.from_id,
                group_id: self.group_id,
                operation: if block {
                    types::MemberJoinRequestOperation::IgnoreAndBlock
                } else {
                    types::MemberJoinRequestOperation::Ignore
                },
                message: "",
            })
            .await
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BotInvitedJoinGroupRequestEvent {
    pub event_id: i64,
    pub from_id: i64,
    #[serde(rename = "nick")]
    pub from_nickname: String,
    pub group_id: i64,
    pub group_name: String,
}

impl BotInvitedJoinGroupRequestEvent {
    pub fn from(&self) -> UserHandle {
        Bot.get_user(self.from_id)
    }

    pub fn group(&self) -> GroupHandle {
        Bot.get_group(self.group_id)
    }

    pub async fn accept<S: MahSession + ?Sized>(&self, session: &S) -> Result<(), S::Error> {
        session
            .handle_bot_invited_join_group_request(&types::HandleBotInvitedJoinGroupRequestArgs {
                event_id: self.event_id,
                from_id: self.from_id,
                group_id: self.group_id,
                operation: types::BotInvitedJoinGroupRequestOperation::Accept,
            })
            .await
    }

    pub async fn ignore<S: MahSession + ?Sized>(&self, session: &S) -> Result<(), S::Error> {
        session
            .handle_bot_invited_join_group_request(&types::HandleBotInvitedJoinGroupRequestArgs {
                event_id: self.event_id,
                from_id: self.from_id,
                group_id: self.group_id,
                operation: types::BotInvitedJoinGroupRequestOperation::Ignore,
            })
            .await
    }
}

#[derive(Clone, Debug)]
pub struct CommandExecutedEvent {
    pub name: String,
    pub args: Vec<IncomingMessageNode>,
    pub source: CommandSource,
}

impl<'de> Deserialize<'de> for CommandExecutedEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Debug, Deserialize)]
        struct Impl {
            name: String,
            args: IncomingMessageContents,
            #[serde(flatten)]
            source: CommandSource,
        }

        let event = Impl::deserialize(deserializer)?;
        Ok(Self {
            name: event.name,
            args: event.args.nodes,
            source: event.source,
        })
    }
}

#[derive(Clone, Debug)]
pub enum CommandSource {
    Friend(FriendDetails),
    Member(MemberDetails),
    Console,
}

impl<'de> Deserialize<'de> for CommandSource {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error;

        #[derive(Debug, Deserialize)]
        pub struct Impl {
            friend: Option<FriendDetails>,
            member: Option<MemberDetails>,
        }

        let value = Impl::deserialize(deserializer)?;
        match (value.friend, value.member) {
            (Some(_), Some(_)) => Err(D::Error::custom(
                "at most one of `friend` and `member` can be present",
            )),
            (Some(friend), None) => Ok(Self::Friend(friend)),
            (None, Some(member)) => Ok(Self::Member(member)),
            (None, None) => Ok(Self::Console),
        }
    }
}

#[derive(Clone, Debug)]
#[enum_dispatch(AnyEvent)]
pub enum Event {
    BotOnline(BotOnlineEvent),
    BotOfflineActive(BotOfflineActiveEvent),
    BotOfflineForced(BotOfflineForcedEvent),
    BotOfflineDropped(BotOfflineDroppedEvent),
    BotRelogin(BotReloginEvent),
    BotMute(BotMuteEvent),
    BotUnmute(BotUnmuteEvent),
    BotJoinGroup(BotJoinGroupEvent),
    BotLeaveGroupActive(BotLeaveGroupActiveEvent),
    BotLeaveGroupKicked(BotLeaveGroupKickedEvent),
    BotLeaveGroupDisband(BotLeaveGroupDisbandEvent),
    BotPermissionChange(BotPermissionChangeEvent),
    StrangerNudge(StrangerNudgeEvent),
    FriendMessageRecall(FriendMessageRecallEvent),
    FriendNudge(FriendNudgeEvent),
    FriendAdd(FriendAddEvent),
    FriendDelete(FriendDeleteEvent),
    FriendNicknameChange(FriendNicknameChangeEvent),
    FriendTyping(FriendTypingEvent),
    GroupMessageRecall(GroupMessageRecallEvent),
    GroupNudge(GroupNudgeEvent),
    GroupNameChange(GroupNameChangeEvent),
    GroupMuteAll(GroupMuteAllEvent),
    GroupAllowAnonymousChat(GroupAllowAnonymousChatEvent),
    GroupAllowConfessTalk(GroupAllowConfessTalkEvent),
    GroupAllowMemberInvite(GroupAllowMemberInviteEvent),
    MemberMute(MemberMuteEvent),
    MemberUnmute(MemberUnmuteEvent),
    MemberJoin(MemberJoinEvent),
    MemberLeaveActive(MemberLeaveActiveEvent),
    MemberLeaveKicked(MemberLeaveKickedEvent),
    MemberNameChange(MemberNameChangeEvent),
    MemberSpecialTitleChange(MemberSpecialTitleChangeEvent),
    MemberPermissionChange(MemberPermissionChangeEvent),
    MemberHonorChange(MemberHonorChangeEvent),
    OtherClientOnline(OtherClientOnlineEvent),
    OtherClientOffline(OtherClientOfflineEvent),
    NewFriendRequest(NewFriendRequestEvent),
    MemberJoinRequest(MemberJoinRequestEvent),
    BotInvitedJoinGroupRequest(BotInvitedJoinGroupRequestEvent),
    CommandExecuted(CommandExecutedEvent),
}

#[enum_dispatch]
#[allow(dead_code)]
trait AnyMessageOrEvent {}

#[derive(Clone, Debug)]
#[enum_dispatch(AnyMessageOrEvent)]
pub enum MessageOrEvent {
    Message(Message),
    Event(Event),
}

impl<'de> Deserialize<'de> for MessageOrEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Debug, Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct NudgeEvent {
            from_id: i64,
            target: i64,
            subject: Subject,
            action: String,
            suffix: String,
        }

        #[derive(Debug, Deserialize)]
        #[serde(tag = "kind")]
        enum Subject {
            Friend(FriendDetails),
            Group(GroupDetails),
            Stranger(StrangerDetails),
        }

        #[derive(Debug, Deserialize)]
        #[serde(tag = "type")]
        enum Impl {
            FriendMessage(FriendMessage),
            FriendSyncMessage(FriendSyncMessage),
            GroupMessage(GroupMessage),
            GroupSyncMessage(GroupSyncMessage),
            TempMessage(TempMessage),
            TempSyncMessage(TempSyncMessage),
            StrangerMessage(StrangerMessage),
            StrangerSyncMessage(StrangerSyncMessage),
            OtherClientMessage(OtherClientMessage),
            BotOnlineEvent(BotOnlineEvent),
            BotOfflineEventActive(BotOfflineActiveEvent),
            BotOfflineEventForce(BotOfflineForcedEvent),
            BotOfflineEventDropped(BotOfflineDroppedEvent),
            BotReloginEvent(BotReloginEvent),
            GroupRecallEvent(GroupMessageRecallEvent),
            FriendRecallEvent(FriendMessageRecallEvent),
            BotGroupPermissionChangeEvent(BotPermissionChangeEvent),
            BotMuteEvent(BotMuteEvent),
            BotUnmuteEvent(BotUnmuteEvent),
            BotJoinGroupEvent(BotJoinGroupEvent),
            BotLeaveEventActive(BotLeaveGroupActiveEvent),
            BotLeaveEventKick(BotLeaveGroupKickedEvent),
            BotLeaveEventDisband(BotLeaveGroupDisbandEvent),
            GroupNameChangeEvent(GroupNameChangeEvent),
            GroupMuteAllEvent(GroupMuteAllEvent),
            GroupAllowAnonymousChatEvent(GroupAllowAnonymousChatEvent),
            GroupAllowConfessTalkEvent(GroupAllowConfessTalkEvent),
            GroupAllowMemberInviteEvent(GroupAllowMemberInviteEvent),
            MemberJoinEvent(MemberJoinEvent),
            MemberLeaveEventKick(MemberLeaveKickedEvent),
            MemberLeaveEventQuit(MemberLeaveActiveEvent),
            MemberCardChangeEvent(MemberNameChangeEvent),
            MemberSpecialTitleChangeEvent(MemberSpecialTitleChangeEvent),
            MemberPermissionChangeEvent(MemberPermissionChangeEvent),
            MemberMuteEvent(MemberMuteEvent),
            MemberUnmuteEvent(MemberUnmuteEvent),
            NewFriendRequestEvent(NewFriendRequestEvent),
            MemberJoinRequestEvent(MemberJoinRequestEvent),
            BotInvitedJoinGroupRequestEvent(BotInvitedJoinGroupRequestEvent),
            NudgeEvent(NudgeEvent),
            FriendInputStatusChangedEvent(FriendTypingEvent),
            FriendNickChangedEvent(FriendNicknameChangeEvent),
            MemberHonorChangeEvent(MemberHonorChangeEvent),
            OtherClientOnlineEvent(OtherClientOnlineEvent),
            OtherClientOfflineEvent(OtherClientOfflineEvent),
            CommandExecutedEvent(CommandExecutedEvent),
            FriendAddEvent(FriendAddEvent),
            FriendDeleteEvent(FriendDeleteEvent),
        }

        Ok(match Impl::deserialize(deserializer)? {
            Impl::FriendMessage(message) => Self::Message(message.into()),
            Impl::FriendSyncMessage(message) => Self::Message(message.into()),
            Impl::GroupMessage(message) => Self::Message(message.into()),
            Impl::GroupSyncMessage(message) => Self::Message(message.into()),
            Impl::TempMessage(message) => Self::Message(message.into()),
            Impl::TempSyncMessage(message) => Self::Message(message.into()),
            Impl::StrangerMessage(message) => Self::Message(message.into()),
            Impl::StrangerSyncMessage(message) => Self::Message(message.into()),
            Impl::OtherClientMessage(message) => Self::Message(message.into()),
            Impl::BotOnlineEvent(event) => Self::Event(event.into()),
            Impl::BotOfflineEventActive(event) => Self::Event(event.into()),
            Impl::BotOfflineEventForce(event) => Self::Event(event.into()),
            Impl::BotOfflineEventDropped(event) => Self::Event(event.into()),
            Impl::BotReloginEvent(event) => Self::Event(event.into()),
            Impl::GroupRecallEvent(event) => Self::Event(event.into()),
            Impl::FriendRecallEvent(event) => Self::Event(event.into()),
            Impl::BotGroupPermissionChangeEvent(event) => Self::Event(event.into()),
            Impl::BotMuteEvent(event) => Self::Event(event.into()),
            Impl::BotUnmuteEvent(event) => Self::Event(event.into()),
            Impl::BotJoinGroupEvent(event) => Self::Event(event.into()),
            Impl::BotLeaveEventActive(event) => Self::Event(event.into()),
            Impl::BotLeaveEventKick(event) => Self::Event(event.into()),
            Impl::BotLeaveEventDisband(event) => Self::Event(event.into()),
            Impl::GroupNameChangeEvent(event) => Self::Event(event.into()),
            Impl::GroupMuteAllEvent(event) => Self::Event(event.into()),
            Impl::GroupAllowAnonymousChatEvent(event) => Self::Event(event.into()),
            Impl::GroupAllowConfessTalkEvent(event) => Self::Event(event.into()),
            Impl::GroupAllowMemberInviteEvent(event) => Self::Event(event.into()),
            Impl::MemberJoinEvent(event) => Self::Event(event.into()),
            Impl::MemberLeaveEventKick(event) => Self::Event(event.into()),
            Impl::MemberLeaveEventQuit(event) => Self::Event(event.into()),
            Impl::MemberCardChangeEvent(event) => Self::Event(event.into()),
            Impl::MemberSpecialTitleChangeEvent(event) => Self::Event(event.into()),
            Impl::MemberPermissionChangeEvent(event) => Self::Event(event.into()),
            Impl::MemberMuteEvent(event) => Self::Event(event.into()),
            Impl::MemberUnmuteEvent(event) => Self::Event(event.into()),
            Impl::NewFriendRequestEvent(event) => Self::Event(event.into()),
            Impl::MemberJoinRequestEvent(event) => Self::Event(event.into()),
            Impl::BotInvitedJoinGroupRequestEvent(event) => Self::Event(event.into()),
            Impl::NudgeEvent(event) => Self::Event(match event.subject {
                Subject::Friend(friend) => FriendNudgeEvent {
                    context: friend,
                    from_id: event.from_id,
                    to_id: event.target,
                    action: event.action,
                    suffix: event.suffix,
                }
                .into(),
                Subject::Group(group) => GroupNudgeEvent {
                    context: group,
                    from_id: event.from_id,
                    to_id: event.target,
                    action: event.action,
                    suffix: event.suffix,
                }
                .into(),
                Subject::Stranger(stranger) => StrangerNudgeEvent {
                    context: stranger,
                    from_id: event.from_id,
                    to_id: event.target,
                    action: event.action,
                    suffix: event.suffix,
                }
                .into(),
            }),
            Impl::FriendInputStatusChangedEvent(event) => Self::Event(event.into()),
            Impl::FriendNickChangedEvent(event) => Self::Event(event.into()),
            Impl::MemberHonorChangeEvent(event) => Self::Event(event.into()),
            Impl::OtherClientOnlineEvent(event) => Self::Event(event.into()),
            Impl::OtherClientOfflineEvent(event) => Self::Event(event.into()),
            Impl::CommandExecutedEvent(event) => Self::Event(event.into()),
            Impl::FriendAddEvent(event) => Self::Event(event.into()),
            Impl::FriendDeleteEvent(event) => Self::Event(event.into()),
        })
    }
}
