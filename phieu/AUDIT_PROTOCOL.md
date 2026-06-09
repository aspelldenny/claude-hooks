# AUDIT_PROTOCOL — Periodic quality audit (RRI-T-lite)

> **Cảm hứng:** RRI-T Methodology v1.0 (Vietnamese Enterprise Software). Kit này dùng phiên bản scope nhỏ — thuần solo / B2C, không enterprise.
> **Khi nào dùng file này:** chu kỳ audit định kỳ (mỗi 5-10 phiếu hoặc 1 wave xong), KHÔNG phải mỗi phiếu. Per-phiếu QA dùng `TEST_CASES_template.md` + Discovery Report.
> **Vai trò trong sos-kit:** layer thứ 3 của QA, sau (1) Per-phiếu test cases và (2) Per-phiếu Discovery Report. Audit phát hiện *gap accumulated* qua nhiều phiếu mà từng cái riêng lẻ không thấy.

---

## 1. Mục đích — tại sao thêm 1 layer nữa

Per-phiếu Discovery Report cho thấy **từng phiếu** có khớp spec không. Nhưng khi 5-10 phiếu ship liên tiếp, có những vấn đề chỉ lộ ra ở scale tổng thể:

- Voice drift dần qua nhiều phiếu (mỗi phiếu lệch 1 chút, tổng cộng lệch nhiều)
- UX rot — feature "chạy" nhưng dùng đau (Save mất 8 giây không feedback)
- Missing features user cần nhưng spec chưa cover (no auto-save, no undo)
- Cross-feature interaction breaks (phiếu A và phiếu B đều pass nhưng combo gãy)
- Tiếng Việt edge cases (dấu, font, currency) — dễ skip ở phiếu lẻ

Audit lite chu kỳ catch những thứ này **trước khi user phát hiện**.

---

## 2. Khi nào chạy audit (5 timing options)

| Trigger | Scope | Effort |
|---|---|---|
| **Sau 5-10 phiếu** | Lite — 3 personas × 4 dimensions, 20-30 test cases | 4-6h |
| **Sau 1 wave/sprint xong** | Lite + cross-wave regression | 6-8h |
| **Trước major release** (v0.X → v0.X+1) | Full — 5 personas × 7 dimensions | 1-2 ngày |
| **Sau incident/regression** | Targeted — chỉ dimension liên quan | 2-4h |
| **Monthly** (tùy chọn) | Smoke + regression suite tự động | 1-2h |

**Default:** Lite mỗi 5-10 phiếu. Wave end = bắt buộc. Pre-major-release = bắt buộc full.

---

## 3. 4-result model — thay binary pass/fail

| Result | Ký hiệu | Ý nghĩa | Ví dụ |
|---|---|---|---|
| **PASS** | ✅ | Đúng spec VÀ tốt cho user | Save → thành công, có feedback |
| **FAIL** | ❌ | Sai spec hoặc không hoạt động | Save → lỗi 500 |
| **PAINFUL** | ⚠️ | Hoạt động nhưng UX kém | Save → mất 8 giây, không feedback |
| **MISSING** | ▢ | Thiếu feature user cần | Không có auto-save |

**Tại sao quan trọng:** binary pass/fail không capture được PAINFUL (UX rot) và MISSING (spec gap). Đây là 2 nguyên nhân chính người dùng bỏ app mà metrics sống không bắt được.

**Action map:**
- PASS → ghi nhận, không cần làm gì
- FAIL → bug ticket → phiếu fix
- PAINFUL → improvement ticket → backlog (Open backlog với tag `[PAINFUL]`)
- MISSING → feedback về requirement / vision → BACKLOG.md hoặc Discovery cho Architect biết

---

## 4. 3 personas (lite — không 5)

RRI-T full có 5 personas. Solo + B2C product chỉ cần 3 thường xuyên:

| Persona | Tư duy | Dimension primary | Khi nào dùng |
|---|---|---|---|
| **End User** | "Tôi dùng hằng ngày" | D1 UI/UX, D7 Edge Cases | LUÔN — default |
| **QA Destroyer** | "Tôi sẽ phá" | D7 Edge Cases, D5 Data | LUÔN — default |
| **Security Auditor** | "Ai có thể lạm dụng?" | D4 Security | CHỈ khi audit động đến auth/payment/PII |

**Bỏ:**
- *Business Analyst* — solo founder + Sếp đã review business rules trong phiếu, không cần persona riêng
- *DevOps Tester* — solo deploy 1 VPS, monitoring đơn giản, không cần persona riêng (per-incident triage đủ)

---

## 5. 4 dimensions (lite — không 7)

| Dim | Mục tiêu | Key Metric | Khi nào audit |
|---|---|---|---|
| **D1 UI/UX** | Giao diện trực quan, responsive, accessible | 0 visual deviation, layout không vỡ | LUÔN — default |
| **D2 API** | Endpoint đúng contract, error graceful | 100% match contract, error code đúng | Khi backend touched |
| **D5 Data Integrity** | Data đúng, đầy đủ, nhất quán | Roundtrip 100%, no data loss | Khi schema/migration touched |
| **D7 Edge Cases** | Xử lý mọi tình huống bất thường | Graceful, no crash | LUÔN — default |

**Bỏ:**
- *D3 Performance* — chỉ cần khi user phàn nàn lag / metric vượt budget
- *D4 Security* — chỉ Security audit mode (xem Persona 3)
- *D6 Infrastructure* — solo deploy, monitoring đủ, không cần audit dim

---

## 6. 4 stress axes (lite — không 8)

Tarot-relevant only:

| Axis | Câu hỏi | Test scenario |
|---|---|---|
| **TIME** | Rapid clicks, deadline gấp | Bulk ops, debounce, double-click |
| **DATA** | Data lớn / nhiều rows | Large list rendering, pagination edge |
| **ERROR** | Save xong phát hiện sai? | Undo / redo, recovery from interruption |
| **LOCALE** | Tiếng Việt có dấu, VND, timezone | Diacritics search, date GMT+7, font rendering |

**Bỏ (rare cho B2C):** COLLAB (multi-user edit), EMERGENCY (workflow đột xuất), SECURITY (đã có persona riêng), INFRA (zero-downtime deploy).

---

## 7. Test case format Q→A→R→P→T

Mỗi test case 1 record với 7 field:

```
ID:        [MODULE]-[D<N>]-[NUMBER]      vd: ONBOARD-D1-007
Persona:   [👤 User | 🔍 QA | 🔒 Sec]
Q:         [Câu hỏi từ góc nhìn persona]
A:         [Expected behavior — 1 câu]
R:         [Requirement extract — REQ-XXX hoặc "phiếu Pxxx"]
P:         [P0 | P1 | P2]
T (Test):
  Precondition: [Setup state]
  Steps: 1...  2...  3...
  Expected: [Kết quả chi tiết]
  Dimension: [D1-D7]
  Stress: [TIME | DATA | ERROR | LOCALE | none]
Result:    [✅ | ❌ | ⚠️ | ▢]  + Notes nếu có
```

**Ví dụ thực:**

```
ID:        TAROT-D1-007
Persona:   👤 End User
Q:         Đang đọc Deep Reading 15 phút, chuyển tab. Quay lại, chat history còn?
A:         Conversation phải còn, scroll position giữ
R:         REQ-Deep-002: Persist conversation across navigation
P:         P0
T (Test):
  Precondition: User logged in, đang Deep Reading turn 5
  Steps:
    1. Edit chat input 10 chars
    2. Mở tab khác, chờ 30s
    3. Quay lại tab Deep Reading
    4. Re-open conversation
  Expected: 5 turns vẫn hiển thị + draft 10 chars trong input box + scroll ở turn 5
  Dimension: D1 (UI/UX) + D5 (Data Integrity)
  Stress: ERROR (recovery from interruption)
Result:    ⚠️ PAINFUL — turns còn nhưng draft 10 chars mất, scroll về top
Notes:     PAINFUL → improvement ticket, không phải fail technical
```

---

## 8. Vietnamese-specific 13 checks (bắt buộc cho B2C VN)

Audit luôn phải qua 13 check sau, dù phiếu không cố tình touch i18n:

| # | Area | Test Case | Expected |
|---|---|---|---|
| 1 | Dấu tiếng Việt | Search "nguyen" tìm "Nguyễn"? | Yes — diacritic-insensitive |
| 2 | Unicode sorting | Sort: Ấn, Bình, Cường, Đức | Đúng thứ tự VN (Đ sau D) |
| 3 | VND Currency | Hiển thị 1234567 VND | 1.234.567 ₫ |
| 4 | Phone format | +84 912 345 678 hoặc 0912345678 | Accept cả 2, normalize |
| 5 | Date format | Hiển thị date | 23/02/2025 (DD/MM/YYYY) |
| 6 | Timezone | Server UTC, display local | GMT+7 consistently |
| 7 | Địa chỉ VN | Vietnamese address | Số nhà/Đường/Phường/Quận/TP |
| 8 | CCCD/CMND | 12 digits CCCD, 9 digits CMND | Accept cả 2 format |
| 9 | Mã số thuế | MST 10 hoặc 13 digits | Validate đúng format |
| 10 | Text overflow | VN text ~30% dài hơn EN | UI không vỡ layout |
| 11 | Font rendering | Diacritics ở small font 10px | Dấu rõ ràng, không cắt |
| 12 | Input methods | Telex, VNI, VIQR | Smooth input |
| 13 | PDF/Image export | Export với Vietnamese content | Dấu đúng (regression P026 share PNG!) |

**Tarot note:** check #13 là chính xác P026 đã ship hôm 26/04 — share PNG variable font crash. Audit thường catch trước khi user complain.

---

## 9. 5 phases — compact version

| # | Phase | Mô tả | Thời gian (lite) | Output |
|---|---|---|---|---|
| 1 | **PREPARE** | Pick scope (phiếu nào trong wave này?), pick personas, pick dimensions, setup test data | 30 phút | Audit Plan section |
| 2 | **DISCOVER** | 3 personas × 4 dim → liệt kê 20-30 test ideas (không full 100-140) | 1-2h | Raw Test Cases list |
| 3 | **STRUCTURE** | Phân loại Impact × Likelihood (xem §10) → priority P0/P1/P2 → viết format Q→A→R→P→T | 1-2h | Structured Test Suite |
| 4 | **EXECUTE** | Chạy P0 trước, P1 sau, ghi 4-result | 2-3h | Test Results với 4-result |
| 5 | **ANALYZE** | Tính coverage per dim, check release gate, generate Audit Report | 30 phút - 1h | `docs/AUDIT_<wave>.md` |

**Tổng lite: 4-8h.** RRI-T full: 1-2 ngày.

---

## 10. Priority — Impact × Likelihood matrix

```
                Unlikely    Possible    Likely
Critical Impact   P1          P0          P0
Major Impact      P2          P1          P0
Minor Impact      P3          P2          P1
```

**Trong audit lite:** chạy hết P0 + P1, P2 chọn lọc, P3 skip (log MISSING vào backlog).

---

## 11. Coverage release gate

```
Coverage = (PASS count) / (Total test cases) × 100%

≥ 85%  → 🟢 Green  → Wave/release approved
70-84% → 🟡 Yellow → Conditional — document known issues, ship với caveat
< 70%  → 🔴 Red    → Block — fix critical (FAIL P0) trước
```

**Release criteria** (Tarot/B2C lite version):
- Tất cả 4 dimensions ≥ 70% PASS
- Ít nhất 3/4 dimension ≥ 85%
- 0 items P0 ở trạng thái FAIL

PAINFUL không block release — log thành improvement ticket trong BACKLOG.md `Open backlog` với tag `[PAINFUL]`. MISSING tương tự, nhưng tag `[GAP]` để Architect biết spec thiếu.

---

## 12. Integration với debate flow (sos-kit v2.1)

Audit **KHÔNG** là phiếu thông thường. KHÔNG cần Architect ↔ Worker debate. Lý do: audit là *measurement*, không phải *change*. Architect không có gì để debate vì không ai propose code change ở đây.

**Flow audit:**

```
Sếp (hoặc orchestrator detect "đã 8 phiếu rồi") → trigger audit
   → main session spawn @agent-worker (AUDIT mode — special prompt)
       → Worker đọc N phiếu gần nhất + DISCOVERIES.md
       → Worker chạy 5 phases (PREPARE → DISCOVER → STRUCTURE → EXECUTE → ANALYZE)
       → Worker append `docs/AUDIT_<wave>.md` với coverage matrix + 4-result table
       → Worker return về main session
   → main session present tóm tắt cho Sếp (AskUserQuestion):
       - Coverage status (Green/Yellow/Red)
       - Top 3 PAINFUL → tạo improvement ticket?
       - Top 3 MISSING → feed về Architect để spec phiếu?
   → Sếp duyệt từng action
```

**Trigger phrase cho Worker AUDIT mode:** `"Worker audit wave X — RRI-T lite"`. Worker nhận diện AUDIT mode khác CHALLENGE và EXECUTE — read-only, không sửa code, không commit, chỉ Write file `docs/AUDIT_*.md`.

---

## 13. Audit report template (để Worker fill)

File: `docs/AUDIT_<wave-name>.md`. Skeleton:

```markdown
# Audit Report — <Wave name / N phiếu range>

> **Trigger:** <sau N phiếu / wave end / pre-release / incident>
> **Scope:** <phiếu nào trong audit này>
> **Date:** <YYYY-MM-DD>
> **Duration:** <thực tế bao lâu>

## 1. Audit plan (Phase 1 — PREPARE)
- Personas dùng: 👤 User + 🔍 QA + [🔒 Sec nếu có]
- Dimensions dùng: D1 + D2 + D5 + D7 + [D4 nếu có]
- Stress axes: TIME + DATA + ERROR + LOCALE

## 2. Test cases (Phase 2-3 — DISCOVER + STRUCTURE)
*(20-30 cases, mỗi case format Q→A→R→P→T)*

### TC-1: <ID>
[Q→A→R→P→T format]

### TC-2: <ID>
...

## 3. Execution results (Phase 4 — EXECUTE)

| ID | Persona | Dim | P | Result | Notes |
|---|---|---|---|---|---|
| TC-1 | User | D1 | P0 | ✅ | — |
| TC-2 | QA | D7 | P0 | ⚠️ | PAINFUL — Save mất 5s |
| ... | | | | | |

## 4. Coverage matrix (Phase 5 — ANALYZE)

| Dim | Tests | ✅ | ❌ | ⚠️ | ▢ | Coverage |
|---|---|---|---|---|---|---|
| D1 UI/UX | 8 | 7 | 0 | 1 | 0 | 88% 🟢 |
| D2 API | 6 | 5 | 1 | 0 | 0 | 83% 🟡 |
| D5 Data | 5 | 5 | 0 | 0 | 0 | 100% 🟢 |
| D7 Edge | 7 | 5 | 0 | 1 | 1 | 71% 🟡 |
| **TỔNG** | **26** | **22** | **1** | **2** | **1** | **85%** 🟢 |

## 5. Action items

### FAIL → bug tickets (urgent)
- [ ] TC-X: <issue> → tạo phiếu `fix/P<NNN>-<slug>`

### PAINFUL → improvement (vào BACKLOG)
- [ ] TC-Y: <UX rot> → BACKLOG.md `Open backlog` `[PAINFUL]`

### MISSING → spec gap (về Architect)
- [ ] TC-Z: <missing feature> → BACKLOG.md `[GAP]` + note Architect ở phiếu kế tiếp

### Vietnamese checks (13 items)
*(Liệt kê check nào fail / painful / missing)*

## 6. Release gate decision

- Coverage status: 🟢 Green / 🟡 Yellow / 🔴 Red
- Decision: [Release approved / Conditional with caveat / Block — fix first]
- Sếp's sign-off: ⏳ pending / ✅ approved on <date>
```

---

## 14. Ai chạy audit

| Role | Trách nhiệm |
|---|---|
| **Sếp** | Trigger audit (mỗi N phiếu), pick scope, nghiệm thu Audit Report, decide actions từ FAIL/PAINFUL/MISSING |
| **Main session (orchestrator)** | Detect trigger nếu cron-like; spawn Worker AUDIT mode; present tóm tắt; route action items vào BACKLOG |
| **Worker subagent (AUDIT mode)** | Chạy 5 phases, write `docs/AUDIT_*.md`. Read-only — KHÔNG sửa code, KHÔNG commit |
| **Architect** | KHÔNG tham gia audit run. Đọc Audit Report khi viết phiếu kế tiếp (giống cách đọc DISCOVERIES.md) |

---

## 15. Anti-pattern (tránh)

1. **Chạy full RRI-T (5×7×8) cho mỗi audit** → 1-2 ngày, kiệt sức, sẽ skip lần sau. Lite first, full chỉ khi pre-major-release.
2. **Audit mỗi phiếu** → overhead lớn, redundant với Discovery Report. Chỉ audit định kỳ, không từng phiếu.
3. **Skip 4-result, dùng binary** → mất signal PAINFUL + MISSING. Bắt buộc 4-result.
4. **Chạy audit mà không chạy Vietnamese 13 checks** → gặp regression P026-style trên production.
5. **Audit Report không có action item** → tài liệu chết. Mỗi FAIL/PAINFUL/MISSING phải có owner + ticket plan.
6. **Worker AUDIT mode "tiện thì sửa luôn"** → vi phạm read-only. Chỉ write `docs/AUDIT_*.md`, không sửa code.

---

## 16. Khác biệt với RRI-T full

| Khía cạnh | RRI-T full (PDF) | RRI-T lite (file này) |
|---|---|---|
| Scope | Vietnamese Enterprise Software | Solo B2C Vietnamese app |
| Personas | 5 (User, BA, QA, DevOps, Security) | 3 (User, QA, Security) |
| Dimensions | 7 (UI/UX, API, Perf, Sec, Data, Infra, Edge) | 4 (UI/UX, API, Data, Edge) — Sec/Perf on demand |
| Stress axes | 8 | 4 (Time, Data, Error, Locale) |
| Test cases | 100-140 / module | 20-30 / wave |
| Effort | 1-2 ngày | 4-8h |
| Khi nào dùng full | Pre-major-release (v0.X → v0.X+1) hoặc incident lớn | KHÔNG default |

---

## 17. Roadmap mở rộng

- **P010 candidate**: viết AUDIT_TEMPLATE.md cụ thể cho nhanh fill (ngoài skeleton ở §13)
- **P011 candidate**: extend `agents/worker.md` với AUDIT mode (sau khi CHALLENGE / EXECUTE đã chạy ổn)
- **P012 candidate**: orchestrator auto-detect trigger "đã N phiếu" → AskUserQuestion "chạy audit không?"
- **P013 candidate**: Vietnamese 13-checks → CI gate (chạy auto pre-deploy)

---

*Inspired by RRI-T Methodology v1.0 (Vietnamese Enterprise Software) — scope nhỏ lại cho solo / B2C.*
