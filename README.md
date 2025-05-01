# oura-sync

Simple standalone CLI / Docker container to sync Oura data to influxdb.

## Metrics

- [x] Heart Rate (`heart.bpm`)
- [x] Cardiovascular Age (Daily)
  - `cardiovascular_age.vascular_age`
- [x] Readiness (Daily)
  - `readiness.score`
  - `readiness.activity_balance`
  - `readiness.body_temperature`
  - `readiness.hrv_balance`
  - `readiness.previous_day_activity`
  - `readiness.previous_night`
  - `readiness.recovery_index`
  - `readiness.resting_heart_rate`
  - `readiness.sleep_balance`
  - `readiness.temperature_deviation`
  - `readiness.temperature_trend_deviation`
- [x] Sleep Routes
  - `sleep.average_breath`
  - `sleep.average_heart_rate`
  - `sleep.average_hrv`
  - `sleep.awake_time`
  - `sleep.bedtime_end`
  - `sleep.bedtime_start`
  - `sleep.efficiency`
  - `sleep.latency`
  - `sleep.light_sleep_duration`
  - `sleep.deep_sleep_duration`
  - `sleep.rem_sleep_duration`
  - `sleep.restless_periods`
  - `sleep.total_sleep_duration`
  - `sleep.time_in_bed`
  - `sleep.sleep_algorithm_version`
  - `sleep.sleep_score_delta`
  - `sleep.sleep_phase_5_min`
  - `sleep.type`
- [ ] Resilience (Daily)
  - `resilience.level` "limited"
  - `resilience.sleep_recovery`
  - `resilience.daytime_recovery`
  - `resilience.stress`
- [ ] Sleep Score (Daily)
  - `sleep.score`
  - `sleep.id`
  - `sleep.deep_sleep`
  - `sleep.efficiency`
  - `sleep.latency`
  - `sleep.rem_sleep`
  - `sleep.restfulness`
  - `sleep.timing`
  - `sleep.total_sleep`
- [ ] Spo2 (Daily)
  - `spo2.average`
  - `spo2.breathing_disturbance_index`
- [ ] Stress (Daily)
  - `stress.id`
  - `stress.stress_high`
  - `stress.recovery_high`
  - `stress.day_summary` "restored"
- [ ] Enhanced Tags
- [ ] Rest Mode Period

## Alternatives

- [nitobuendia/oura-custom-component](https://github.com/nitobuendia/oura-custom-component) - to home assistant (python)
- [leshy/oura_sync](https://github.com/leshy/oura_sync/tree/main) - to influxdb (typescript)
