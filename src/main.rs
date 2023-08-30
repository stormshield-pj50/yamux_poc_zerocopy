use std::fmt::Debug;
use zerocopy::byteorder::network_endian::{U16, U32};
use zerocopy::{AsBytes, ByteSlice, ByteSliceMut, FromBytes, FromZeroes, Ref};

#[derive(Copy, Clone, Debug, FromZeroes, FromBytes, AsBytes)]
#[repr(packed)]
pub struct Header<T> {
    version: Version,
    tag: u8,
    flags: Flags,
    stream_id: StreamId,
    length: Len,
    _marker: std::marker::PhantomData<T>,
}

#[derive(Copy, Clone, Debug)]
pub enum Tag {
    Data,
    WindowUpdate,
    Ping,
    GoAway,
}

impl TryFrom<u8> for Tag {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Data),
            1 => Ok(Self::WindowUpdate),
            2 => Ok(Self::Ping),
            3 => Ok(Self::GoAway),
            _ => Err(()),
        }
    }
}

#[derive(Copy, Clone, Debug, FromZeroes, FromBytes, AsBytes)]
#[repr(C)]
pub struct Version(u8);

#[derive(Copy, Clone, Debug, FromZeroes, FromBytes, AsBytes)]
#[repr(C)]
pub struct Len(U32);

impl Len {
    pub fn val(self) -> u32 {
        self.0.get()
    }
}

#[derive(Copy, Clone, Debug, FromZeroes, FromBytes, AsBytes)]
#[repr(C)]
pub struct StreamId(U32);

impl StreamId {
    pub(crate) fn new(val: u32) -> Self {
        StreamId(val.into())
    }

    pub fn val(self) -> u32 {
        self.0.get()
    }
}

#[derive(Debug)]
struct Frame<B: ByteSlice, T: Debug + Copy> {
    header: Ref<B, Header<T>>,
    body: B,
}

impl<B: ByteSlice, T: Debug + Copy> Frame<B, T> {
    fn parse(bytes: B) -> Option<Frame<B, T>> {
        let (header, body) = Ref::new_from_prefix(bytes)?;
        let frame = Frame { header, body };
        if <u8 as TryInto<Tag>>::try_into(frame.header.tag).is_ok() {
            Some(frame)
        } else {
            None
        }
    }

    fn version(&self) -> Version {
        self.header.version
    }

    fn length(&self) -> Len {
        self.header.length
    }
}

impl<B: ByteSliceMut, T: Debug + Copy> Frame<B, T> {
    fn set_tag(&mut self, tag: Tag) {
        self.header.tag = tag as u8;
    }
}

#[derive(Copy, Clone, Debug, FromZeroes, FromBytes, AsBytes)]
#[repr(C)]
pub struct Flags(U16);

#[derive(Copy, Clone, Debug)]
pub struct Data {}

impl Header<Data> {
    /// Create a new data frame header.
    pub fn data(id: StreamId, len: u32) -> Self {
        Header {
            version: Version(0),
            tag: Tag::Data as u8,
            flags: Flags(0.into()),
            stream_id: id,
            length: Len(len.into()),
            _marker: std::marker::PhantomData,
        }
    }
}

fn main() {
    // Parse some bytes into a frame
    let mut bytes = [
        0x01, 0x02, 0x01, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x03, 0x01, 0x02, 0x03,
    ];
    let mut frame = Frame::<&mut [u8], Data>::parse(&mut bytes[..]).unwrap();
    println!("{frame:?}");
    println!(
        "frame version = {:?}, frame length = {:?}",
        frame.version(),
        frame.length()
    );
    println!("frame body = {:?}", frame.body);

    // Get frame's whole bytes
    println!("{:?}", frame.header.bytes());

    // Update frame
    frame.set_tag(Tag::GoAway);

    // Get frame's whole bytes
    println!("{:?}", frame.header.bytes());
}
