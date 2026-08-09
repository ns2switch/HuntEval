SELECT event_id, principal, event_time, event_name FROM aws_cloudtrail WHERE source_ip = '203.0.113.77' ORDER BY event_time, event_id;
