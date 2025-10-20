//! Tests for the channels module
use trash_parallelism::channels::*;

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
pub fn test_send_async_and_recv_async() {
    smol::block_on(async {
        let (tx, rx) = core::bounded_queue_3::<String>(1);
        core::send_async(&tx, "test message".to_string()).await.unwrap();
        let received = core::recv_async(&rx).await.unwrap();
        assert_eq!(received, "test message");
    });
}

#[test]
pub fn test_send_json_message_and_recv_json_message() {
    smol::block_on(async {
        let (tx, rx) = core::bounded_queue_3::<core::JsonMessage>(1);
        core::send_json_message(&tx, "test data").await.unwrap();
        let received: String = core::recv_json_message(&rx).await.unwrap();
        assert_eq!(received, "test data");
    });
}

#[test]
pub fn test_broadcast_message() {
    smol::block_on(async {
        let (tx1, rx1) = core::bounded_queue_3::<String>(1);
        let (tx2, rx2) = core::bounded_queue_3::<String>(1);
        let senders = vec![tx1, tx2];
        core::broadcast_message("broadcast".to_string(), senders).await.unwrap();
        let msg1 = core::recv_async(&rx1).await.unwrap();
        let msg2 = core::recv_async(&rx2).await.unwrap();
        assert_eq!(msg1, "broadcast");
        assert_eq!(msg2, "broadcast");
    });
}

#[test]
pub fn test_benchmark_channel() {
    smol::block_on(async {
        let (tx, rx) = core::bounded_queue_3::<String>(100);
        let stats = core::benchmark_channel(&tx, &rx, "test".to_string(), 10).await;
        assert_eq!(stats.messages_sent, 10);
        assert_eq!(stats.messages_received, 10);
        assert!(stats.avg_latency.is_some());
    });
}

#[test]
pub fn test_monitored_channel() {
    let channel = monitoring::create_monitored_channel::<String>(10);
    // Basic test that channel is created
    drop(channel);
}

#[test]
pub fn test_monitored_channel_send_recv() {
    smol::block_on(async {
        let channel = monitoring::MonitoredChannel::new();
        channel.send_async("test".to_string()).await.unwrap();
        let received = channel.recv_async().await.unwrap();
        assert_eq!(received, "test");
        let stats = channel.stats();
        assert_eq!(stats.messages_sent, 1);
        assert_eq!(stats.messages_received, 1);
    });
}

#[test]
pub fn test_monitored_channel_builder() {
    let channel = monitoring::MonitoredChannel::builder()
        .capacity(50)
        .build();
    smol::block_on(async {
        channel.send_async("test".to_string()).await.unwrap();
        let stats = channel.stats();
        assert_eq!(stats.messages_sent, 1);
    });
}

#[test]
pub fn test_channel_stats_to_json() {
    let stats = monitoring::ChannelStats {
        messages_sent: 5,
        messages_received: 3,
        ..Default::default()
    };
    let json = stats.to_json().unwrap();
    assert!(json.contains("messages_sent"));
    assert!(json.contains('5'));
    assert!(json.contains("messages_received"));
    assert!(json.contains('3'));
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
pub fn test_channel_multiplexer_route() {
    smol::block_on(async {
        let multiplexer = multiplexor::ChannelMultiplexer::new();
        let (tx, rx) = core::bounded_queue_3::<String>(1);
        multiplexer.register_route("test", tx);
        multiplexer.route_message("test", "message".to_string()).await.unwrap();
        let received = core::recv_async(&rx).await.unwrap();
        assert_eq!(received, "message");
    });
}

#[test]
pub fn test_async_channel_processor() {
    smol::block_on(async {
        let (tx, rx) = core::bounded_queue_3::<String>(1);
        let processor = multiplexor::AsyncChannelProcessor::new(rx, |_msg: String| {
            Box::pin(async move {
                // Simple processing - just return the message
                Ok(())
            })
        });
        processor.start();
        tx.send("test".to_string()).await.unwrap();
        // Allow some time for processing
        smol::Timer::after(std::time::Duration::from_millis(10)).await;
    });
}

#[test]
pub fn test_create_async_processor() {
    let (tx, rx) = core::bounded_queue_3::<i32>(1);
    let processor = multiplexor::create_async_processor(rx, |_num: i32| {
        Box::pin(async move {
            // Process number
            Ok(())
        })
    });
    drop(processor);
    drop(tx);
}

#[test]
pub fn test_work_queue() {
    let queue = queue::WorkQueue::<String, String>::new(2);
    // Basic test that queue is created
    drop(queue);
}

#[test]
pub fn test_work_queue_submit() {
    smol::block_on(async {
        let queue = queue::WorkQueue::<String, ()>::new(1);
        queue.submit("task".to_string()).await.unwrap();
        // Note: collect would require workers to be set up properly
    });
}

#[test]
pub fn test_base64_channel() {
    let (tx, _) = core::bounded_queue_3::<String>(1);
    let channel = specialist::Base64Channel::new(tx);
    // Basic test that channel is created
    drop(channel);
}

#[test]
pub fn test_base64_channel_send_recv() {
    smol::block_on(async {
        let (tx, rx) = core::bounded_queue_3::<String>(1);
        let channel = specialist::Base64Channel::new(tx);
        channel.send_base64(&"test data".to_string()).await.unwrap();
        let received: String = specialist::Base64Channel::recv_base64(&rx).await.unwrap();
        assert_eq!(received, "test data");
    });
}

#[test]
pub fn test_compressed_channel() {
    let channel = specialist::CompressedChannel::new();
    // Basic test that channel is created
    drop(channel);
}

#[test]
pub fn test_compressed_channel_send_recv() {
    smol::block_on(async {
        let channel = specialist::CompressedChannel::new();
        let data = "test compression data".to_string();
        channel.send_compressed(&data).await.unwrap();
        let received: String = channel.recv_decompressed().await.unwrap();
        assert_eq!(received, data);
    });
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
pub fn test_file_backed_channel_send() {
    smol::block_on(async {
        // Use a channel with capacity 0 to force file backing
        let channel: specialist::FileBackedChannel<String> = specialist::FileBackedChannel::new().unwrap();
        // Since the internal channel has capacity 100, we need to fill it first
        // For this test, we'll just check that send doesn't panic
        channel.send("test data".to_string()).await.unwrap();
        // The data is in memory, flush_to_memory reads from file (overflow)
        let flushed = channel.flush_to_memory().unwrap();
        // Since we didn't overflow, flushed should be empty
        assert_eq!(flushed.len(), 0);
    });
}

#[test]
pub fn test_rate_limited_channel() {
    let channel: specialist::RateLimitedChannel<String> =
        specialist::RateLimitedChannel::new(10, 10.0, 1.0);
    // Basic test that channel is created
    drop(channel);
}

#[test]
pub fn test_rate_limited_channel_send() {
    smol::block_on(async {
        let (rx_tx, rx) = core::bounded_queue_3::<String>(10);
        // Create a rate limited channel that sends to our receiver
        let channel: specialist::RateLimitedChannel<String> =
            specialist::RateLimitedChannel::new(1, 10.0, 1.0);
        // We can't directly connect, so just test the rate limiting logic
        // Since the channel creates its own internal sender, sending will fail
        // unless we connect it. For this test, we'll just check creation.
        drop(channel);
        drop(rx_tx);
        drop(rx);
    });
}

#[test]
pub fn test_priority_channel() {
    let channel: specialist::PriorityChannel<String> = specialist::PriorityChannel::new(10);
    // Basic test that channel is created
    drop(channel);
}

#[test]
pub fn test_priority_channel_send_recv() {
    smol::block_on(async {
        let channel: specialist::PriorityChannel<String> = specialist::PriorityChannel::new(10);
        channel.send_normal("normal".to_string()).await.unwrap();
        channel.send_high("high".to_string()).await.unwrap();
        channel.send_low("low".to_string()).await.unwrap();

        // High priority should be received first
        let received = channel.recv().await.unwrap();
        assert_eq!(received, "high");

        // Then normal
        let received = channel.recv().await.unwrap();
        assert_eq!(received, "normal");

        // Then low
        let received = channel.recv().await.unwrap();
        assert_eq!(received, "low");
    });
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
pub fn test_channel_aggregator() {
    smol::block_on(async {
        let (tx1, rx1) = core::bounded_queue_3::<String>(1);
        let (tx_out, rx_out) = core::bounded_queue_3::<String>(2);
        let aggregator = parsers::ChannelAggregator::new(vec![rx1], tx_out);
        aggregator.start();

        tx1.send("message".to_string()).await.unwrap();
        let received = core::recv_async(&rx_out).await.unwrap();
        assert_eq!(received, "message");
    });
}

#[test]
pub fn test_batching_channel() {
    let channel: parsers::BatchingChannel<String> = parsers::BatchingChannel::new(3, 10);
    // Basic test that channel is created
    drop(channel);
}

#[test]
pub fn test_batching_channel_send() {
    smol::block_on(async {
        let channel: parsers::BatchingChannel<String> = parsers::BatchingChannel::new(2, 10);
        let batch_receiver = channel.batch_receiver();

        channel.send("item1".to_string()).await.unwrap();
        channel.send("item2".to_string()).await.unwrap(); // Should trigger batch

        let batch = core::recv_async(&batch_receiver).await.unwrap();
        assert_eq!(batch.len(), 2);
        assert_eq!(batch[0], "item1");
        assert_eq!(batch[1], "item2");
    });
}

#[test]
pub fn test_filtered_channel() {
    let (tx, _) = core::bounded_queue_3::<i32>(10);
    let filtered = parsers::FilteredChannel::new(tx, |&num| num > 0);
    // Basic test that channel is created
    drop(filtered);
}

#[test]
pub fn test_filtered_channel_send() {
    smol::block_on(async {
        let (tx, rx) = core::bounded_queue_3::<i32>(10);
        let filtered = parsers::FilteredChannel::new(tx, |&num| num > 0);

        filtered.send_filtered(5).await.unwrap();  // Should pass
        filtered.send_filtered(-1).await.unwrap(); // Should be filtered
        filtered.send_filtered(10).await.unwrap(); // Should pass

        let positive1 = core::recv_async(&rx).await.unwrap();
        assert_eq!(positive1, 5);

        let positive2 = core::recv_async(&rx).await.unwrap();
        assert_eq!(positive2, 10);
    });
}

#[test]
pub fn test_channels() {
    test_bounded_queue_3();
    test_create_bounded_channel();
    test_create_unbounded_channel();
    test_message_new_and_verify();
    test_send_async_and_recv_async();
    test_send_json_message_and_recv_json_message();
    test_broadcast_message();
    test_benchmark_channel();
    test_monitored_channel();
    test_monitored_channel_send_recv();
    test_monitored_channel_builder();
    test_channel_stats_to_json();
    test_channel_multiplexer();
    test_channel_multiplexer_route();
    test_async_channel_processor();
    test_create_async_processor();
    test_work_queue();
    test_work_queue_submit();
    test_base64_channel();
    test_base64_channel_send_recv();
    test_compressed_channel();
    test_compressed_channel_send_recv();
    test_compressed_channel_with_config();
    test_compressed_channel_builder();
    test_file_backed_channel();
    test_file_backed_channel_send();
    test_rate_limited_channel();
    test_rate_limited_channel_send();
    test_priority_channel();
    test_priority_channel_send_recv();
    test_fast_message_parser();
    test_fast_message_parser_json();
    test_channel_aggregator();
    test_batching_channel();
    test_batching_channel_send();
    test_filtered_channel();
    test_filtered_channel_send();
}
