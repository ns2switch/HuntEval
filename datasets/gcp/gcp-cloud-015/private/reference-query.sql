SELECT event_id, principal, event_time, event_name FROM gcp_audit WHERE source_ip = '203.0.113.85' AND user_agent = 'unrecognized-client/2.0' ORDER BY event_time, event_id;
