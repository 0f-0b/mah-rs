use std::borrow::Cow;
use std::fmt;
use std::time::{Duration, SystemTime};

use derive_into_owned::IntoOwned;
use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

use crate::{
    types, Bot, FileHandle, FriendDetails, GroupDetails, GroupHandle, MemberDetails, MemberHandle,
    MessageHandle, OtherClientDetails, StrangerDetails, UserHandle,
};

#[enum_dispatch]
#[allow(dead_code)]
trait AnyIncomingMessageNode {}

#[enum_dispatch]
#[allow(dead_code)]
trait AnyOutgoingMessageNode {}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct AtNode {
    #[serde(rename = "target")]
    pub target_id: i64,
}

impl AtNode {
    pub fn target(&self) -> UserHandle {
        Bot.get_user(self.target_id)
    }
}

impl From<&AtNode> for AtNode {
    fn from(value: &AtNode) -> Self {
        *value
    }
}

pub fn at(target_id: i64) -> AtNode {
    AtNode { target_id }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct AtAllNode {}

impl From<&AtAllNode> for AtAllNode {
    fn from(value: &AtAllNode) -> Self {
        *value
    }
}

pub fn at_all() -> AtAllNode {
    AtAllNode {}
}

#[derive(Clone, Debug, Deserialize)]
pub struct IncomingFaceNode {
    #[serde(rename = "faceId")]
    pub id: i32,
    pub name: String,
    #[serde(rename = "isSuperFace")]
    pub super_face: bool,
}

#[derive(Clone, Debug, IntoOwned, Serialize)]
pub struct OutgoingFaceNode<'a> {
    #[serde(flatten)]
    pub face: OutgoingFace<'a>,
    #[serde(rename = "isSuperFace")]
    pub super_face: bool,
}

#[derive(Clone, Debug, IntoOwned, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OutgoingFace<'a> {
    #[serde(rename = "faceId")]
    Id(i32),
    Name(Cow<'a, str>),
}

impl<'a> OutgoingFaceNode<'a> {
    pub fn to_super_face(self, super_face: bool) -> Self {
        Self {
            face: self.face,
            super_face,
        }
    }
}

impl From<&IncomingFaceNode> for OutgoingFaceNode<'static> {
    fn from(value: &IncomingFaceNode) -> Self {
        Self {
            face: OutgoingFace::Id(value.id),
            super_face: value.super_face,
        }
    }
}

pub fn face_from_id(id: i32) -> OutgoingFaceNode<'static> {
    OutgoingFaceNode {
        face: OutgoingFace::Id(id),
        super_face: false,
    }
}

pub fn face_from_name<'a>(name: impl Into<Cow<'a, str>>) -> OutgoingFaceNode<'a> {
    OutgoingFaceNode {
        face: OutgoingFace::Name(name.into()),
        super_face: false,
    }
}

#[derive(Clone, Debug, IntoOwned, Deserialize, Serialize)]
pub struct PlainNode<'a> {
    pub text: Cow<'a, str>,
}

impl<'a> From<&'a PlainNode<'a>> for PlainNode<'a> {
    fn from(value: &'a PlainNode<'a>) -> Self {
        Self {
            text: Cow::Borrowed(&value.text),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IncomingImageNode {
    pub image_id: String,
    pub url: String,
    pub width: i32,
    pub height: i32,
    pub size: i64,
    pub image_type: ImageType,
    pub is_emoji: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ImageType {
    Png,
    Bmp,
    Jpg,
    Gif,
    Apng,
    Unknown,
}

#[derive(Clone, Debug, IntoOwned, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OutgoingImageNode<'a> {
    ImageId(Cow<'a, str>),
    Url(Cow<'a, str>),
    Path(Cow<'a, str>),
    Base64(Cow<'a, str>),
}

impl<'a> From<&'a IncomingImageNode> for OutgoingImageNode<'a> {
    fn from(value: &'a IncomingImageNode) -> Self {
        Self::ImageId(Cow::Borrowed(&value.image_id))
    }
}

pub fn image_from_id<'a>(id: impl Into<Cow<'a, str>>) -> OutgoingImageNode<'a> {
    OutgoingImageNode::ImageId(id.into())
}

pub fn image_from_url<'a>(url: impl Into<Cow<'a, str>>) -> OutgoingImageNode<'a> {
    OutgoingImageNode::Url(url.into())
}

pub fn image_from_path<'a>(path: impl Into<Cow<'a, str>>) -> OutgoingImageNode<'a> {
    OutgoingImageNode::Path(path.into())
}

pub fn image_from_base64<'a>(base64: impl Into<Cow<'a, str>>) -> OutgoingImageNode<'a> {
    OutgoingImageNode::Base64(base64.into())
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IncomingVoiceNode {
    pub voice_id: String,
    pub url: String,
    #[serde(rename = "length")]
    pub length_secs: i64,
}

impl IncomingVoiceNode {
    pub fn length(&self) -> Duration {
        Duration::from_secs(self.length_secs as u64)
    }
}

#[derive(Clone, Debug, IntoOwned, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OutgoingVoiceNode<'a> {
    VoiceId(Cow<'a, str>),
    Url(Cow<'a, str>),
    Path(Cow<'a, str>),
    Base64(Cow<'a, str>),
}

pub fn voice_from_id<'a>(id: impl Into<Cow<'a, str>>) -> OutgoingVoiceNode<'a> {
    OutgoingVoiceNode::VoiceId(id.into())
}

pub fn voice_from_url<'a>(url: impl Into<Cow<'a, str>>) -> OutgoingVoiceNode<'a> {
    OutgoingVoiceNode::Url(url.into())
}

pub fn voice_from_path<'a>(path: impl Into<Cow<'a, str>>) -> OutgoingVoiceNode<'a> {
    OutgoingVoiceNode::Path(path.into())
}

pub fn voice_from_base64<'a>(base64: impl Into<Cow<'a, str>>) -> OutgoingVoiceNode<'a> {
    OutgoingVoiceNode::Base64(base64.into())
}

#[derive(Clone, Debug, IntoOwned, Deserialize, Serialize)]
pub struct XmlNode<'a> {
    #[serde(rename = "xml")]
    pub contents: Cow<'a, str>,
}

impl<'a> From<&'a XmlNode<'a>> for XmlNode<'a> {
    fn from(value: &'a XmlNode<'a>) -> Self {
        Self {
            contents: Cow::Borrowed(&value.contents),
        }
    }
}

pub fn xml<'a>(contents: impl Into<Cow<'a, str>>) -> XmlNode<'a> {
    XmlNode {
        contents: contents.into(),
    }
}

#[derive(Clone, Debug, IntoOwned, Serialize)]
pub struct OutgoingJsonNode<'a> {
    #[serde(rename = "json")]
    pub contents: Cow<'a, str>,
}

pub fn json<'a>(contents: impl Into<Cow<'a, str>>) -> OutgoingJsonNode<'a> {
    OutgoingJsonNode {
        contents: contents.into(),
    }
}

#[derive(Clone, Debug, IntoOwned, Deserialize, Serialize)]
pub struct AppNode<'a> {
    #[serde(rename = "content")]
    pub contents: Cow<'a, str>,
}

impl<'a> From<&'a AppNode<'a>> for AppNode<'a> {
    fn from(value: &'a AppNode<'a>) -> Self {
        Self {
            contents: Cow::Borrowed(&value.contents),
        }
    }
}

pub fn app<'a>(contents: impl Into<Cow<'a, str>>) -> AppNode<'a> {
    AppNode {
        contents: contents.into(),
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct IncomingMarketFaceNode {
    pub id: i32,
    pub name: String,
}

#[derive(Clone, Debug, IntoOwned, Deserialize, Serialize)]
pub struct PokeNode<'a> {
    pub name: Cow<'a, str>,
}

impl<'a> From<&'a PokeNode<'a>> for PokeNode<'a> {
    fn from(value: &'a PokeNode<'a>) -> Self {
        Self {
            name: Cow::Borrowed(&value.name),
        }
    }
}

pub fn poke<'a>(name: impl Into<Cow<'a, str>>) -> PokeNode<'a> {
    PokeNode { name: name.into() }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct DiceNode {
    pub value: i32,
}

impl From<&DiceNode> for DiceNode {
    fn from(value: &DiceNode) -> Self {
        *value
    }
}

pub fn dice(value: i32) -> DiceNode {
    DiceNode { value }
}

#[derive(Clone, Debug, IntoOwned, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicShareNode<'a> {
    pub kind: Cow<'a, str>,
    pub title: Cow<'a, str>,
    pub summary: Cow<'a, str>,
    pub jump_url: Cow<'a, str>,
    pub picture_url: Cow<'a, str>,
    pub music_url: Cow<'a, str>,
    pub brief: Cow<'a, str>,
}

impl<'a> From<&'a MusicShareNode<'a>> for MusicShareNode<'a> {
    fn from(value: &'a MusicShareNode<'a>) -> Self {
        Self {
            kind: Cow::Borrowed(&value.kind),
            title: Cow::Borrowed(&value.title),
            summary: Cow::Borrowed(&value.summary),
            jump_url: Cow::Borrowed(&value.jump_url),
            picture_url: Cow::Borrowed(&value.picture_url),
            music_url: Cow::Borrowed(&value.music_url),
            brief: Cow::Borrowed(&value.brief),
        }
    }
}

pub fn music_share<'a>(
    kind: impl Into<Cow<'a, str>>,
    title: impl Into<Cow<'a, str>>,
    summary: impl Into<Cow<'a, str>>,
    jump_url: impl Into<Cow<'a, str>>,
    picture_url: impl Into<Cow<'a, str>>,
    music_url: impl Into<Cow<'a, str>>,
    brief: impl Into<Cow<'a, str>>,
) -> MusicShareNode<'a> {
    MusicShareNode {
        kind: kind.into(),
        title: title.into(),
        summary: summary.into(),
        jump_url: jump_url.into(),
        picture_url: picture_url.into(),
        music_url: music_url.into(),
        brief: brief.into(),
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct IncomingForwardNode {
    #[serde(rename = "nodeList")]
    pub messages: Vec<IncomingForwardedMessage>,
}

#[derive(Clone, Debug)]
pub struct IncomingForwardedMessage {
    pub sender_id: i64,
    pub sender_name: String,
    pub time: i32,
    pub quote: Option<QuotedMessage>,
    pub nodes: Vec<IncomingMessageNode>,
}

impl IncomingForwardedMessage {
    pub fn sender(&self) -> UserHandle {
        Bot.get_user(self.sender_id)
    }
}

impl<'de> Deserialize<'de> for IncomingForwardedMessage {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Debug, Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Impl {
            sender_id: i64,
            time: i32,
            sender_name: String,
            message_chain: IncomingMessageContents,
        }

        let message = Impl::deserialize(deserializer)?;
        Ok(Self {
            sender_id: message.sender_id,
            sender_name: message.sender_name,
            time: message.time,
            quote: message.message_chain.quote,
            nodes: message.message_chain.nodes,
        })
    }
}

#[derive(Clone, Debug, IntoOwned, Serialize)]
pub struct OutgoingForwardNode<'a> {
    #[serde(rename = "nodeList")]
    pub messages: Vec<OutgoingForwardedMessage<'a>>,
    pub display: Option<ForwardDisplay<'a>>,
}

#[enum_dispatch]
#[allow(dead_code)]
trait AnyOutgoingForwardedMessage {}

#[derive(Clone, Copy, Debug)]
pub struct RefForwardedMessage {
    pub context: Option<i64>,
    pub id: i32,
}

impl From<MessageHandle> for RefForwardedMessage {
    fn from(value: MessageHandle) -> Self {
        Self {
            context: Some(value.context),
            id: value.id,
        }
    }
}

#[derive(Clone, Debug, IntoOwned)]
pub struct CustomForwardedMessage<'a> {
    pub sender_id: i64,
    pub sender_name: Cow<'a, str>,
    pub time: Option<i32>,
    pub nodes: Vec<OutgoingMessageNode<'a>>,
}

#[derive(Clone, Debug, IntoOwned)]
#[enum_dispatch(AnyOutgoingForwardedMessage)]
pub enum OutgoingForwardedMessage<'a> {
    Ref(RefForwardedMessage),
    Custom(CustomForwardedMessage<'a>),
}

impl From<MessageHandle> for OutgoingForwardedMessage<'_> {
    fn from(value: MessageHandle) -> Self {
        Self::Ref(value.into())
    }
}

impl<'a> Serialize for OutgoingForwardedMessage<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Debug, Serialize)]
        struct Id {
            message_id: i32,
        }

        #[derive(Debug, Serialize)]
        struct Ref {
            message_ref: types::MessageIdArgs,
        }

        #[derive(Debug, Serialize)]
        struct Custom<'a> {
            sender_id: i64,
            #[serde(skip_serializing_if = "Option::is_none")]
            time: Option<i32>,
            sender_name: &'a str,
            message_chain: &'a [OutgoingMessageNode<'a>],
        }

        match self {
            Self::Ref(message) => {
                if let Some(context) = message.context {
                    Ref {
                        message_ref: types::MessageIdArgs {
                            target: context,
                            message_id: message.id,
                        },
                    }
                    .serialize(serializer)
                } else {
                    Id {
                        message_id: message.id,
                    }
                    .serialize(serializer)
                }
            }
            Self::Custom(message) => Custom {
                sender_id: message.sender_id,
                time: message.time,
                sender_name: &message.sender_name,
                message_chain: &message.nodes,
            }
            .serialize(serializer),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ForwardDisplay<'a> {
    pub brief: Option<Cow<'a, str>>,
    pub preview: Option<Vec<Cow<'a, str>>>,
    pub source: Option<Cow<'a, str>>,
    pub summary: Option<Cow<'a, str>>,
    pub title: Option<Cow<'a, str>>,
}

impl<'a> ForwardDisplay<'a> {
    pub fn into_owned(self) -> ForwardDisplay<'static> {
        ForwardDisplay {
            brief: self.brief.map(|val| val.into_owned().into()),
            preview: self
                .preview
                .map(|val| val.into_iter().map(|x| x.into_owned().into()).collect()),
            source: self.source.map(|val| val.into_owned().into()),
            summary: self.summary.map(|val| val.into_owned().into()),
            title: self.title.map(|val| val.into_owned().into()),
        }
    }
}

impl<'a> TryFrom<&'a IncomingForwardNode> for OutgoingForwardNode<'a> {
    type Error = TryIntoOutgoingError;

    fn try_from(value: &'a IncomingForwardNode) -> Result<Self, Self::Error> {
        Ok(Self {
            messages: value
                .messages
                .iter()
                .map(|message| {
                    Ok(OutgoingForwardedMessage::Custom(CustomForwardedMessage {
                        sender_id: message.sender_id,
                        sender_name: Cow::Borrowed(&message.sender_name),
                        time: Some(message.time),
                        nodes: message
                            .nodes
                            .iter()
                            .map(|node| node.try_into())
                            .collect::<Result<_, _>>()?,
                    }))
                })
                .collect::<Result<_, _>>()?,
            display: None,
        })
    }
}

pub fn forward<'a>(
    messages: impl IntoIterator<Item = impl Into<OutgoingForwardedMessage<'a>>>,
    display: Option<ForwardDisplay<'a>>,
) -> OutgoingForwardNode<'a> {
    OutgoingForwardNode {
        messages: messages.into_iter().map(Into::into).collect(),
        display,
    }
}

fn deserialize_file_id<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    let mut id = String::deserialize(deserializer)?;
    if !id.starts_with('/') {
        id.insert(0, '/')
    }
    Ok(id)
}

#[derive(Clone, Debug, Deserialize)]
pub struct IncomingFileNode {
    #[serde(deserialize_with = "deserialize_file_id")]
    pub id: String,
    pub name: String,
    pub size: i64,
}

impl IncomingFileNode {
    pub fn file(&self, group: GroupHandle) -> FileHandle {
        group.get_file(self.id.clone())
    }

    pub fn into_file(self, group: GroupHandle) -> FileHandle {
        group.get_file(self.id)
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IncomingShortVideoNode {
    pub video_id: String,
    #[serde(rename = "filename")]
    pub name: String,
    #[serde(rename = "fileSize")]
    pub size: i64,
    #[serde(rename = "fileFormat")]
    pub video_type: String,
    #[serde(rename = "videoUrl")]
    pub url: Option<String>,
    #[serde(rename = "fileMd5")]
    pub md5: String,
}

#[derive(Clone, Debug, IntoOwned, Serialize)]
pub struct OutgoingMiraiCodeNode<'a> {
    pub code: Cow<'a, str>,
}

pub fn mirai_code<'a>(code: impl Into<Cow<'a, str>>) -> OutgoingMiraiCodeNode<'a> {
    OutgoingMiraiCodeNode { code: code.into() }
}

#[derive(Clone, Debug)]
#[enum_dispatch(AnyIncomingMessageNode)]
pub enum IncomingMessageNode {
    At(AtNode),
    AtAll(AtAllNode),
    Face(IncomingFaceNode),
    Plain(PlainNode<'static>),
    Image(IncomingImageNode),
    Voice(IncomingVoiceNode),
    Xml(XmlNode<'static>),
    App(AppNode<'static>),
    Poke(PokeNode<'static>),
    Dice(DiceNode),
    MarketFace(IncomingMarketFaceNode),
    MusicShare(MusicShareNode<'static>),
    Forward(IncomingForwardNode),
    File(IncomingFileNode),
    ShortVideo(IncomingShortVideoNode),
}

#[derive(Clone, Debug, IntoOwned, Serialize)]
#[enum_dispatch(AnyOutgoingMessageNode)]
#[serde(tag = "type")]
pub enum OutgoingMessageNode<'a> {
    At(AtNode),
    AtAll(AtAllNode),
    Face(OutgoingFaceNode<'a>),
    Plain(PlainNode<'a>),
    Image(OutgoingImageNode<'a>),
    Voice(OutgoingVoiceNode<'a>),
    Xml(XmlNode<'a>),
    Json(OutgoingJsonNode<'a>),
    App(AppNode<'a>),
    Poke(PokeNode<'a>),
    Dice(DiceNode),
    MusicShare(MusicShareNode<'a>),
    Forward(OutgoingForwardNode<'a>),
    MiraiCode(OutgoingMiraiCodeNode<'a>),
}

impl<'a, T: Into<Cow<'a, str>>> From<T> for OutgoingMessageNode<'a> {
    fn from(value: T) -> Self {
        Self::Plain(PlainNode { text: value.into() })
    }
}

impl<'a> TryFrom<&'a IncomingMessageNode> for OutgoingMessageNode<'a> {
    type Error = TryIntoOutgoingError;

    fn try_from(value: &'a IncomingMessageNode) -> Result<Self, Self::Error> {
        match value {
            IncomingMessageNode::At(node) => Ok(Self::At(node.into())),
            IncomingMessageNode::AtAll(node) => Ok(Self::AtAll(node.into())),
            IncomingMessageNode::Face(node) => Ok(Self::Face(node.into())),
            IncomingMessageNode::Plain(node) => Ok(Self::Plain(node.into())),
            IncomingMessageNode::Image(node) => Ok(Self::Image(node.into())),
            IncomingMessageNode::Voice(_) => Err(TryIntoOutgoingError),
            IncomingMessageNode::Xml(node) => Ok(Self::Xml(node.into())),
            IncomingMessageNode::App(node) => Ok(Self::App(node.into())),
            IncomingMessageNode::Poke(node) => Ok(Self::Poke(node.into())),
            IncomingMessageNode::Dice(node) => Ok(Self::Dice(node.into())),
            IncomingMessageNode::MarketFace(_) => Err(TryIntoOutgoingError),
            IncomingMessageNode::MusicShare(node) => Ok(Self::MusicShare(node.into())),
            IncomingMessageNode::Forward(node) => Ok(Self::Forward(node.try_into()?)),
            IncomingMessageNode::File(_) => Err(TryIntoOutgoingError),
            IncomingMessageNode::ShortVideo(_) => Err(TryIntoOutgoingError),
        }
    }
}

#[derive(Clone, Copy, Debug, Error)]
#[error("cannot convert to outgoing message")]
pub struct TryIntoOutgoingError;

#[derive(Clone, Debug)]
pub struct QuotedMessageContents {
    pub id: Option<i32>,
    pub nodes: Vec<IncomingMessageNode>,
}

#[enum_dispatch]
pub trait AnyQuotedMessage {
    fn handle(&self) -> Option<MessageHandle>;
    fn contents(&self) -> &QuotedMessageContents;

    fn id(&self) -> Option<i32> {
        self.contents().id
    }

    fn nodes(&self) -> &[IncomingMessageNode] {
        &self.contents().nodes
    }
}

#[derive(Clone, Debug)]
pub struct QuotedGroupMessage {
    pub context_id: i64,
    pub sender_id: i64,
    pub contents: QuotedMessageContents,
}

impl QuotedGroupMessage {
    pub fn context(&self) -> GroupHandle {
        Bot.get_group(self.context_id)
    }

    pub fn sender(&self) -> MemberHandle {
        self.context().get_member(self.sender_id)
    }
}

impl AnyQuotedMessage for QuotedGroupMessage {
    fn handle(&self) -> Option<MessageHandle> {
        Some(Bot.get_message(self.contents.id?, self.context_id))
    }

    fn contents(&self) -> &QuotedMessageContents {
        &self.contents
    }
}

#[derive(Clone, Debug)]
pub struct QuotedUserMessage {
    pub receiver_id: i64,
    pub sender_id: i64,
    pub contents: QuotedMessageContents,
}

impl QuotedUserMessage {
    pub fn receiver(&self) -> UserHandle {
        Bot.get_user(self.receiver_id)
    }

    pub fn sender(&self) -> UserHandle {
        Bot.get_user(self.sender_id)
    }
}

impl AnyQuotedMessage for QuotedUserMessage {
    fn handle(&self) -> Option<MessageHandle> {
        Some(Bot.get_message(self.contents.id?, self.sender_id))
    }

    fn contents(&self) -> &QuotedMessageContents {
        &self.contents
    }
}

#[derive(Clone, Debug)]
#[enum_dispatch(AnyQuotedMessage)]
pub enum QuotedMessage {
    Group(QuotedGroupMessage),
    User(QuotedUserMessage),
}

#[derive(Clone, Debug)]
pub struct IncomingMessageContents {
    pub id: Option<i32>,
    pub time_secs: Option<i32>,
    pub quote: Option<QuotedMessage>,
    pub nodes: Vec<IncomingMessageNode>,
}

impl IncomingMessageContents {
    pub fn time(&self) -> Option<SystemTime> {
        SystemTime::UNIX_EPOCH.checked_add(Duration::from_secs(self.time_secs? as u64))
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct OutgoingMessageContents<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote: Option<i32>,
    #[serde(rename = "messageChain")]
    pub nodes: &'a [OutgoingMessageNode<'a>],
}

impl<'a> OutgoingMessageContents<'a> {
    pub fn new(nodes: &'a [OutgoingMessageNode<'a>]) -> Self {
        Self { quote: None, nodes }
    }

    pub fn quote(self, quote: Option<MessageHandle>) -> Self {
        self.quote_id(quote.map(|message| message.id))
    }

    pub fn quote_id(self, quote: Option<i32>) -> Self {
        Self { quote, ..self }
    }
}

#[macro_export]
macro_rules! make_message {
  ($($x:expr),* $(,)?) => {{
    $crate::message::OutgoingMessageContents::new(&[
      $($crate::__::Into::into($x),)*
    ])
  }};
}

const _: () = {
    use serde::de::{Error, SeqAccess, Visitor};

    struct IncomingMessageContentsVisitor;

    impl<'de> Visitor<'de> for IncomingMessageContentsVisitor {
        type Value = IncomingMessageContents;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a message chain")
        }

        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            #[derive(Debug, Deserialize)]
            struct IncomingSourceNode {
                id: i32,
                time: i32,
            }

            #[derive(Debug, Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct IncomingQuoteNode {
                id: i32,
                sender_id: i64,
                target_id: i64,
                group_id: i64,
                origin: IncomingMessageContents,
            }

            #[derive(Debug, Deserialize)]
            #[serde(tag = "type")]
            enum Impl {
                Source(IncomingSourceNode),
                At(AtNode),
                AtAll(AtAllNode),
                Face(IncomingFaceNode),
                Plain(PlainNode<'static>),
                Image(IncomingImageNode),
                Voice(IncomingVoiceNode),
                Xml(XmlNode<'static>),
                App(AppNode<'static>),
                Quote(IncomingQuoteNode),
                Poke(PokeNode<'static>),
                Dice(DiceNode),
                MarketFace(IncomingMarketFaceNode),
                MusicShare(MusicShareNode<'static>),
                Forward(IncomingForwardNode),
                File(IncomingFileNode),
                ShortVideo(IncomingShortVideoNode),
            }

            let mut id = None;
            let mut time_secs = None;
            let mut quote = None;
            let mut nodes = Vec::new();
            while let Some(node) = seq.next_element::<Impl>()? {
                match node {
                    Impl::Source(node) => {
                        if time_secs.is_some() {
                            return Err(A::Error::custom("duplicate `Source`"));
                        }
                        id = (node.id != 0).then_some(node.id);
                        time_secs = Some(node.time);
                    }
                    Impl::At(node) => nodes.push(IncomingMessageNode::At(node)),
                    Impl::AtAll(node) => nodes.push(IncomingMessageNode::AtAll(node)),
                    Impl::Face(node) => nodes.push(IncomingMessageNode::Face(node)),
                    Impl::Plain(node) => nodes.push(IncomingMessageNode::Plain(node)),
                    Impl::Image(node) => nodes.push(IncomingMessageNode::Image(node)),
                    Impl::Voice(node) => nodes.push(IncomingMessageNode::Voice(node)),
                    Impl::Xml(node) => nodes.push(IncomingMessageNode::Xml(node)),
                    Impl::App(node) => nodes.push(IncomingMessageNode::App(node)),
                    Impl::Quote(node) => {
                        if quote.is_some() {
                            return Err(A::Error::custom("duplicate `Quote`"));
                        }
                        let contents = QuotedMessageContents {
                            id: (node.id != 0).then_some(node.id),
                            nodes: node.origin.nodes,
                        };
                        quote = Some(if node.group_id == 0 {
                            QuotedMessage::User(QuotedUserMessage {
                                receiver_id: node.target_id,
                                sender_id: node.sender_id,
                                contents,
                            })
                        } else {
                            QuotedMessage::Group(QuotedGroupMessage {
                                context_id: node.target_id,
                                sender_id: node.sender_id,
                                contents,
                            })
                        });
                    }
                    Impl::Poke(node) => nodes.push(IncomingMessageNode::Poke(node)),
                    Impl::Dice(node) => nodes.push(IncomingMessageNode::Dice(node)),
                    Impl::MarketFace(node) => nodes.push(IncomingMessageNode::MarketFace(node)),
                    Impl::MusicShare(node) => nodes.push(IncomingMessageNode::MusicShare(node)),
                    Impl::Forward(node) => nodes.push(IncomingMessageNode::Forward(node)),
                    Impl::File(node) => nodes.push(IncomingMessageNode::File(node)),
                    Impl::ShortVideo(node) => nodes.push(IncomingMessageNode::ShortVideo(node)),
                }
            }
            Ok(IncomingMessageContents {
                id,
                time_secs,
                quote,
                nodes,
            })
        }
    }

    impl<'de> Deserialize<'de> for IncomingMessageContents {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            deserializer.deserialize_seq(IncomingMessageContentsVisitor)
        }
    }
};

#[enum_dispatch]
pub trait AnyMessage {
    fn handle(&self) -> Option<MessageHandle>;
    fn contents(&self) -> &IncomingMessageContents;

    fn id(&self) -> Option<i32> {
        self.contents().id
    }

    fn time(&self) -> Option<SystemTime> {
        self.contents().time()
    }

    fn time_secs(&self) -> Option<i32> {
        self.contents().time_secs
    }

    fn quote(&self) -> Option<&QuotedMessage> {
        self.contents().quote.as_ref()
    }

    fn nodes(&self) -> &[IncomingMessageNode] {
        self.contents().nodes.as_ref()
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct FriendMessage {
    pub sender: FriendDetails,
    #[serde(rename = "messageChain")]
    pub contents: IncomingMessageContents,
}

impl FriendMessage {
    pub fn context(&self) -> &FriendDetails {
        &self.sender
    }

    pub fn sender(&self) -> &FriendDetails {
        &self.sender
    }
}

impl AnyMessage for FriendMessage {
    fn handle(&self) -> Option<MessageHandle> {
        Some(Bot.get_message(self.contents.id?, self.sender.0.id))
    }

    fn contents(&self) -> &IncomingMessageContents {
        &self.contents
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct FriendSyncMessage {
    #[serde(rename = "subject")]
    pub context: FriendDetails,
    #[serde(rename = "messageChain")]
    pub contents: IncomingMessageContents,
}

impl FriendSyncMessage {
    pub fn context(&self) -> &FriendDetails {
        &self.context
    }
}

impl AnyMessage for FriendSyncMessage {
    fn handle(&self) -> Option<MessageHandle> {
        Some(Bot.get_message(self.contents.id?, self.context.0.id))
    }

    fn contents(&self) -> &IncomingMessageContents {
        &self.contents
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct GroupMessage {
    pub sender: MemberDetails,
    #[serde(rename = "messageChain")]
    pub contents: IncomingMessageContents,
}

impl GroupMessage {
    pub fn context(&self) -> &GroupDetails {
        &self.sender.group
    }

    pub fn sender(&self) -> &MemberDetails {
        &self.sender
    }
}

impl AnyMessage for GroupMessage {
    fn handle(&self) -> Option<MessageHandle> {
        Some(Bot.get_message(self.contents.id?, self.sender.group.id))
    }

    fn contents(&self) -> &IncomingMessageContents {
        &self.contents
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct GroupSyncMessage {
    #[serde(rename = "subject")]
    pub context: GroupDetails,
    #[serde(rename = "messageChain")]
    pub contents: IncomingMessageContents,
}

impl GroupSyncMessage {
    pub fn context(&self) -> &GroupDetails {
        &self.context
    }
}

impl AnyMessage for GroupSyncMessage {
    fn handle(&self) -> Option<MessageHandle> {
        Some(Bot.get_message(self.contents.id?, self.context.id))
    }

    fn contents(&self) -> &IncomingMessageContents {
        &self.contents
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct TempMessage {
    pub sender: MemberDetails,
    #[serde(rename = "messageChain")]
    pub contents: IncomingMessageContents,
}

impl TempMessage {
    pub fn context(&self) -> &MemberDetails {
        &self.sender
    }

    pub fn sender(&self) -> &MemberDetails {
        &self.sender
    }
}

impl AnyMessage for TempMessage {
    fn handle(&self) -> Option<MessageHandle> {
        Some(Bot.get_message(self.contents.id?, self.sender.id))
    }

    fn contents(&self) -> &IncomingMessageContents {
        &self.contents
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct TempSyncMessage {
    #[serde(rename = "subject")]
    pub context: MemberDetails,
    #[serde(rename = "messageChain")]
    pub contents: IncomingMessageContents,
}

impl TempSyncMessage {
    pub fn context(&self) -> &MemberDetails {
        &self.context
    }
}

impl AnyMessage for TempSyncMessage {
    fn handle(&self) -> Option<MessageHandle> {
        Some(Bot.get_message(self.contents.id?, self.context.id))
    }

    fn contents(&self) -> &IncomingMessageContents {
        &self.contents
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct StrangerMessage {
    pub sender: StrangerDetails,
    #[serde(rename = "messageChain")]
    pub contents: IncomingMessageContents,
}

impl StrangerMessage {
    pub fn context(&self) -> &StrangerDetails {
        self.sender()
    }

    pub fn sender(&self) -> &StrangerDetails {
        &self.sender
    }
}

impl AnyMessage for StrangerMessage {
    fn handle(&self) -> Option<MessageHandle> {
        Some(Bot.get_message(self.contents.id?, self.sender.0.id))
    }

    fn contents(&self) -> &IncomingMessageContents {
        &self.contents
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct StrangerSyncMessage {
    #[serde(rename = "subject")]
    pub context: StrangerDetails,
    #[serde(rename = "messageChain")]
    pub contents: IncomingMessageContents,
}

impl StrangerSyncMessage {
    pub fn context(&self) -> &StrangerDetails {
        &self.context
    }
}

impl AnyMessage for StrangerSyncMessage {
    fn handle(&self) -> Option<MessageHandle> {
        Some(Bot.get_message(self.contents.id?, self.context.0.id))
    }

    fn contents(&self) -> &IncomingMessageContents {
        &self.contents
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct OtherClientMessage {
    pub sender: OtherClientDetails,
    #[serde(rename = "messageChain")]
    pub contents: IncomingMessageContents,
}

impl OtherClientMessage {
    pub fn context(&self) -> &OtherClientDetails {
        self.sender()
    }

    pub fn sender(&self) -> &OtherClientDetails {
        &self.sender
    }
}

impl AnyMessage for OtherClientMessage {
    fn handle(&self) -> Option<MessageHandle> {
        None
    }

    fn contents(&self) -> &IncomingMessageContents {
        &self.contents
    }
}

#[derive(Clone, Debug, Deserialize)]
#[enum_dispatch(AnyMessage)]
#[serde(tag = "type")]
pub enum Message {
    #[serde(rename = "FriendMessage")]
    Friend(FriendMessage),
    #[serde(rename = "FriendSyncMessage")]
    FriendSync(FriendSyncMessage),
    #[serde(rename = "GroupMessage")]
    Group(GroupMessage),
    #[serde(rename = "GroupSyncMessage")]
    GroupSync(GroupSyncMessage),
    #[serde(rename = "TempMessage")]
    Temp(TempMessage),
    #[serde(rename = "TempSyncMessage")]
    TempSync(TempSyncMessage),
    #[serde(rename = "StrangerMessage")]
    Stranger(StrangerMessage),
    #[serde(rename = "StrangerSyncMessage")]
    StrangerSync(StrangerSyncMessage),
    #[serde(rename = "OtherClientMessage")]
    OtherClient(OtherClientMessage),
}
