use nv_video_codec_sys::CUvideopacketflags;

bitflags! {
    pub struct DecoderPacketFlags: u32 {
        /// Set when a discontinuity has to be signalled
        const DISCONTINUITY = CUvideopacketflags::CUVID_PKT_DISCONTINUITY;
        /// Set when the packet contains exactly one frame or one field
        const END_OF_PICTURE = CUvideopacketflags::CUVID_PKT_ENDOFPICTURE;
        /// Set when this is the last packet for this stream
        const END_OF_STREAM = CUvideopacketflags::CUVID_PKT_ENDOFSTREAM;
        /// If this flag is set along with CUVID_PKT_ENDOFSTREAM, an additional (dummy) display callback will be invoked with null value of CUVIDPARSERDISPINFO which should be interpreted as end of the stream.
        const NOTIFY_EOS = CUvideopacketflags::CUVID_PKT_NOTIFY_EOS;
        /// Timestamp is valid
        const TIMESTAMP = CUvideopacketflags::CUVID_PKT_TIMESTAMP;
    }
}

impl Into<CUvideopacketflags::Type> for DecoderPacketFlags {
    fn into(self) -> CUvideopacketflags::Type {
        self.bits as CUvideopacketflags::Type
    }
}
