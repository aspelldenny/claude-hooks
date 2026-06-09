# PHIẾU P000-genesis: <Project Name>

> **Loại đặc biệt:** Master phiếu — chỉ tồn tại 1 cái duy nhất per project.
> **Filename:** `docs/ticket/P000-genesis.md` (cố định, không qua counter).
> **Branch:** `genesis/P000-bootstrap` (worktree riêng, merge vào main sau khi scaffold xong).
> **Tạo qua `sos contract`** — không dùng `phieu` shell function thường.

---

> **Spec Hash:** `<auto-filled by sos contract — SHA256 of frozen sections>`
> **Locked at:** `<ISO 8601 timestamp>`
> **Stack:** `<inherited from BLUEPRINT.md>`
> **Status:** Draft / Locked / Scaffolded / Launched

---

## 1. Vision Anchor (frozen — vào spec_hash)

### One-liner
[Một câu mô tả sản phẩm — copy từ `docs/PROJECT.md`]

### 3 hard rules (voice / UX / tech invariants)
1. [Rule 1 — VD: "Không xưng 'bạn' trong narrative content"]
2. [Rule 2 — VD: "Không CTA ép hành động — advice ≠ command"]
3. [Rule 3 — VD: "Mọi credit deduction phải atomic transaction"]

### Non-goals (explicit)
- [Cái KHÔNG làm trong MVP — VD: "Không support multi-language"]
- [Cái defer sang phase 2 — VD: "Không có mobile app native"]

---

## 2. MVP Scope (frozen — vào spec_hash)

### Core features (must-have to launch)
- [ ] **F1:** [Tên feature] — [1 dòng what + why]
- [ ] **F2:** ...
- [ ] **F3:** ...

### Can ship without (defer to phase 2)
- ~~F4~~ — defer (ghi lý do nếu cần)
- ~~F5~~ — defer

> **Quy tắc:** Bất kỳ phiếu Pxxx nào sau Genesis muốn thêm deliverable ngoài "Core features" → **reject** (scope creep). Nếu Chủ nhà muốn thêm thật → cập nhật P000, sinh spec_hash mới, audit trail.

---

## 3. Tech Commitments (frozen — vào spec_hash)

### Stack
| Layer | Choice | Reference |
|-------|--------|-----------|
| Framework | [VD: Next.js 15 App Router] | `recipes/framework-starter/nextjs-15-app-router.md` |
| Language | [VD: TypeScript] | — |
| DB | [VD: PostgreSQL 16 + Prisma] | `recipes/infra/docker-compose-postgres.md` |
| Auth | [VD: NextAuth Google + Email] | `recipes/auth/nextauth-google-email.md` |
| Payment | [VD: PayOS VN] | `recipes/payment/payos-vn.md` |
| AI | [VD: Multi-model Opus → Gemini → OpenRouter] | `recipes/ai/multi-model-fallback.md` |
| Observability | [VD: Sentry + Umami self-host] | `recipes/observability/sentry-nextjs.md` + `recipes/observability/umami-self-host.md` |
| Infra | [VD: VPS Docker Compose + Nginx + Certbot] | `recipes/infra/vps-bootstrap-ubuntu.md` |
| CI | [VD: GitHub Actions canary] | `recipes/observability/canary-github-actions.md` |

### Recipes to apply (ordered)
> **[Kiến trúc sư fill]** — Thợ sẽ apply lần lượt qua `sos apply <name>`.

1. `infra/vps-bootstrap-ubuntu`
2. `infra/docker-compose-nginx`
3. `infra/docker-compose-postgres`
4. `framework-starter/nextjs-15-app-router`
5. `auth/nextauth-google-email`
6. `payment/payos-vn`
7. `ai/multi-model-fallback`
8. `observability/sentry-nextjs`
9. `observability/umami-self-host`
10. `observability/canary-github-actions`

### Recipes thiếu (cần forge mới)
> Nếu library chưa có recipe nào trong list, đánh dấu ở đây để Kiến trúc sư chạy `/forge`.

- `[name]` — [lý do cần] — status: `pending forge`

---

## 4. Verification Anchors (Task 0 cho mọi phiếu con)

> **Quy tắc:** Mọi phiếu Pxxx (xxx ≥ 001) đọc bảng này TRƯỚC khi viết Task 0 của riêng nó.
> Chủ nhà điền các invariant + Kiến trúc sư xác minh tồn tại sau khi scaffold xong.

| # | Invariant | Verify command | Expected | Result |
|---|-----------|----------------|----------|--------|
| 1 | Repo có docker-compose.yml | `ls docker-compose.yml` | file exists | — |
| 2 | Có docs/BACKEND_GUIDE.md | `ls docs/BACKEND_GUIDE.md` | file exists | — |
| 3 | Có .env.example với DATABASE_URL | `grep DATABASE_URL .env.example` | match | — |
| 4 | Phiếu workflow đã init | `ls .phieu-counter` | file exists | — |
| 5 | CLAUDE.md có 3-vai instructions | `grep "Chủ nhà" CLAUDE.md` | match | — |
| 6 | docs/DISCOVERIES.md initialized | `ls docs/DISCOVERIES.md` | file exists | — |
| ... | [Project-specific invariants — Kiến trúc sư thêm] | | | |

---

## 5. Launch Checklist (mục đích = gate cho `sos launch`)

> **Source of truth:** `phieu/LAUNCH_CHECKLIST.md` — copy vào đây và tick dần qua các phiếu P001+.
> **Quy tắc:** `sos launch` HARD BLOCK nếu < 100% tick.

### Infra
- [ ] Domain mua + DNS trỏ tới VPS
- [ ] SSL certbot auto-renew
- [ ] VPS firewall + fail2ban
- [ ] Backup cron PostgreSQL daily

### Code health
- [ ] Type-check clean toàn repo
- [ ] Tests cover core features (≥ 1 test/feature)
- [ ] `.env.example` đầy đủ keys
- [ ] CLAUDE.md cho Thợ updated theo stack thực

### Production gates
- [ ] Error tracking (Sentry / equivalent)
- [ ] Analytics (Umami / GA / Plausible)
- [ ] Uptime monitor (Jarvis / UptimeRobot)
- [ ] `/health` endpoint trả 200
- [ ] Rate limiting cho public endpoints
- [ ] Pre-deploy guard pass (`guard check_all`)
- [ ] Canary CI workflow active

### Legal / UX
- [ ] Privacy policy
- [ ] Terms of service
- [ ] First-test-user flow chạy end-to-end
- [ ] Payment flow tested với real money (nếu monetized)

### Docs
- [ ] CHANGELOG.md có entry "v0.1.0 — Launch"
- [ ] DISCOVERIES.md có entry post-launch retro

---

## 6. Relay Rules (Chủ nhà ↔ Kiến trúc sư ↔ Thợ)

### Chỉ Chủ nhà được sửa
- Mục `1. Vision Anchor` (one-liner, hard rules, non-goals)
- Mục `2. MVP Scope > Core features` (move qua "Can ship without" được, nhưng KHÔNG thêm mới giữa chừng)

### Chỉ Kiến trúc sư được sửa
- Mục `3. Tech Commitments` (stack choice, recipes list)
- Mục `4. Verification Anchors` (thêm invariant project-specific)

### Thợ KHÔNG được sửa file này
- Thợ chỉ tick checklist mục 5 sau khi xong từng phiếu Pxxx liên quan.
- Phát hiện assumption sai → ghi vào `docs/DISCOVERIES.md`, **không** sửa P000.

### Spec hash
- Sửa mục 1, 2, 3 → `sos contract --rehash` → spec_hash mới + audit trail trong `.sos/state.toml > history`.
- Sửa mục 4, 5 (verification + checklist) KHÔNG đổi spec_hash (working area).

---

## 7. Audit trail

> Tự động append bởi `sos contract --rehash` mỗi lần re-lock.

| Date | Spec Hash | Changed | By | Reason |
|------|-----------|---------|----|----|
| `<ISO>` | `<hash>` | Initial lock | Chủ nhà | Genesis |
