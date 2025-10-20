//! Tests for the channels module
use trash_utilities::channels::*;

#[test]
pub fn test_bounded_queue_3() {
    let (tx, rx) = core::bounded_queue_3::<String>(5);
    // Basic test that channels are created
    drop(tx);
    drop(rx);
}

#[test]
pub fn test_create_bounded_channel() {
    let (tx, rx) = core::create_bounded_channel::<String>(5);
    // Basic test that channels are created
    drop(tx);
    drop(rx);
}

#[test]
pub fn test_create_unbounded_channel() {
    let (tx, rx) = core::create_unbounded_channel::<String>();
    // Basic test that channels are created
    drop(tx);
    drop(rx);
}

#[test]
pub fn test_message_new_and_verify() {
    let msg = core::Message::new("test data".to_string());
    assert!(msg.verify());
    assert_eq!(msg.payload, "test data");
    assert!(msg.id.starts_with("msg_"));
    assert!(msg.checksum.is_some());
}

#[test]
pub fn test_monitored_channel() {
    let channel = monitoring::create_monitored_channel::<String>(10);
    // Basic test that channel is created
    drop(channel);
}

#[test]
pub fn test_channel_stats() {
    let mut stats = monitoring::ChannelStats::default();
    assert_eq!(stats.messages_sent, 0);
    stats.messages_sent = 10;
    stats.reset();
    assert_eq!(stats.messages_sent, 0);
}

#[test]
pub fn test_channel_multiplexer() {
    let multiplexer = multiplexor::ChannelMultiplexer::new();
    // Basic test that multiplexer is created
    drop(multiplexer);
}

#[test]
pub fn test_work_queue() {
    let queue = queue::WorkQueue::<String, String>::new(2);
    // Basic test that queue is created
    drop(queue);
}

#[test]
pub fn test_base64_channel() {
    let (tx, _) = core::bounded_queue_3::<String>(1);
    let channel = specialist::Base64Channel::new(tx);
    // Basic test that channel is created
    drop(channel);
}

#[test]
pub fn test_compressed_channel() {
    let channel = specialist::CompressedChannel::new();
    // Basic test that channel is created
    drop(channel);
}

#[test]
pub fn test_compressed_channel_with_config() {
    let channel = specialist::CompressedChannel::with_config(50, 9);
    // Basic test that channel is created
    drop(channel);
}

#[test]
pub fn test_compressed_channel_builder() {
    let channel = specialist::CompressedChannel::builder()
        .capacity(200)
        .compression_level(11)
        .build();
    // Basic test that channel is created
    drop(channel);
}

#[test]
pub fn test_file_backed_channel() {
    // Would need temp file, but basic structure test
    // let channel: specialist::FileBackedChannel<String> = specialist::FileBackedChannel::new().unwrap();
}

#[test]
pub fn test_rate_limited_channel() {
    let channel: specialist::RateLimitedChannel<String> =
        specialist::RateLimitedChannel::new(10, 10.0, 1.0);
    // Basic test that channel is created
    drop(channel);
}

#[test]
pub fn test_priority_channel() {
    let channel: specialist::PriorityChannel<String> = specialist::PriorityChannel::new(10);
    // Basic test that channel is created
    drop(channel);
}

#[test]
pub fn test_fast_message_parser() {
    let parser = parsers::FastMessageParser::new('\n');
    let buffer = b"line1\nline2\nline3";
    let messages = parser.parse_messages(buffer);
    assert_eq!(messages.len(), 3);
    assert_eq!(messages[0], b"line1");
    assert_eq!(messages[1], b"line2");
    assert_eq!(messages[2], b"line3");
}

#[test]
pub fn test_fast_message_parser_json() {
    let parser = parsers::FastMessageParser::new('\n');
    let buffer = b"{\"name\":\"Alice\"}\n{\"name\":\"Bob\"}";
    let messages = parser.parse_json_messages(buffer).unwrap();
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0]["name"], "Alice");
    assert_eq!(messages[1]["name"], "Bob");
}

#[test]
pub fn test_batching_channel() {
    let channel: parsers::BatchingChannel<String> = parsers::BatchingChannel::new(3, 10);
    // Basic test that channel is created
    drop(channel);
}

#[test]
pub fn test_filtered_channel() {
    let (tx, _) = core::bounded_queue_3::<i32>(10);
    let filtered = parsers::FilteredChannel::new(tx, |&num| num > 0);
    // Basic test that channel is created
    drop(filtered);
}

#[test]
pub fn test_channels() {
    test_bounded_queue_3();
    test_create_bounded_channel();
    test_create_unbounded_channel();
    test_message_new_and_verify();
    test_monitored_channel();
    test_channel_stats();
    test_channel_multiplexer();
    test_work_queue();
    test_base64_channel();
    test_compressed_channel();
    test_compressed_channel_with_config();
    test_compressed_channel_builder();
    test_rate_limited_channel();
    test_priority_channel();
    test_fast_message_parser();
    test_fast_message_parser_json();
    test_batching_channel();
    test_filtered_channel();
}
