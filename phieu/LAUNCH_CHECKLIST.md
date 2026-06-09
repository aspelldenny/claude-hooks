# Launch Checklist — Solo Dev 0→1 Gate

> **Mục đích:** Một danh sách 20 mục **bắt buộc** trước khi gọi `sos launch`. Hard block nếu chưa 100%.
> **Source:** Tích luỹ từ kinh nghiệm tarot (P004 revenue leak, SOUL-PIVOT credit atomic, SESSION-MODEL-V1 race conditions).
> **Cách dùng:** Copy section này vào `phieu/P000-genesis.md > 5. Launch Checklist`, tick dần qua các phiếu P001+.

---

## Infra (5)

- [ ] **L1.** Domain mua + DNS trỏ tới VPS (Cloudflare hoặc tương đương)
- [ ] **L2.** SSL active (certbot auto-renew test pass — `certbot renew --dry-run`)
- [ ] **L3.** VPS hardening — UFW/iptables firewall + fail2ban + SSH key-only
- [ ] **L4.** Backup cron PostgreSQL daily — verify restore test trên staging
- [ ] **L5.** docker-compose prod stack chạy ổn định ≥ 24h không restart loop

## Code health (4)

- [ ] **L6.** Type-check clean toàn repo (`pnpm type-check` / `cargo check` exit 0)
- [ ] **L7.** Tests cover core features — ≥ 1 test mỗi must-have feature trong P000
- [ ] **L8.** `.env.example` đầy đủ keys, không leak secrets vào git
- [ ] **L9.** CLAUDE.md cho Thợ updated theo stack thực — 3-vai instructions clear

## Production gates (5)

- [ ] **L10.** Error tracking active (Sentry/Glitchtip/equivalent) — test 1 error reach dashboard
- [ ] **L11.** Analytics active (Umami/Plausible/GA) — test 1 pageview reach dashboard
- [ ] **L12.** Uptime monitor active — alerting test pass (Telegram/Slack/email)
- [ ] **L13.** `/health` endpoint trả 200 + check critical deps (DB, Redis, external APIs)
- [ ] **L14.** Rate limiting cho public endpoints — test với 100 req/s không sập

## Pre-deploy & runtime (3)

- [ ] **L15.** Pre-deploy guard pass — `guard check_all` exit 0 (schema drift, env sync, canary)
- [ ] **L16.** Canary CI workflow active — post-deploy health check tự động
- [ ] **L17.** Payment flow tested với **real money small amount** (nếu monetized) — refund + dispute flow rõ

## Legal / UX (2)

- [ ] **L18.** Privacy policy + Terms of service publish
- [ ] **L19.** First-test-user flow chạy end-to-end — signup → core feature → exit clean

## Docs (1)

- [ ] **L20.** CHANGELOG.md có `v0.1.0 — Launch` entry + DISCOVERIES.md entry "pre-launch retro"

---

## Tự bỏ qua được không?

`sos launch` mặc định **hard block** mọi mục chưa tick. Bypass cần:
```bash
sos launch --skip L17,L18  # phải kèm --reason "..."
```
Audit trail vào `.sos/state.toml > history`. **Không recommend** trừ khi sếp tự gánh hậu quả documented.

## Nguồn từ tarot (DNA)

| Mục | Học từ | Lý do thêm |
|-----|--------|-----------|
| L7 | SESSION-MODEL-V1 phiếu | Logic state machine không test → P004 revenue leak |
| L13 | docker-compose 5-service tarot | astro-service down → cả deep reading flow chết |
| L14 | tarot streak abuse 2026-04-15 | Không rate limit → spam streak |
| L17 | tarot PayOS pre-charge | Pre-charge sai → 4 sessions completed mà credits=0 |
| L20 | tarot DISCOVERIES.md culture | Mỗi launch là 1 cột mốc — phải retro để recipe library học |
