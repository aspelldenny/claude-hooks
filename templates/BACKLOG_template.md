# BACKLOG — <Project Name>

> **Mục đích:** Single source of truth cho "Chủ nhà nên làm gì tiếp theo".
> Idea mới → vào đây trước (qua /idea skill). Phiếu → chỉ viết cho item trong Active sprint.
> Wave-based, KHÔNG time-based. Sprint kết thúc khi xong hoặc Chủ nhà đổi hướng.
>
> **Quy tắc Architect (Rule 0):** Architect chỉ viết phiếu cho item nằm trong "Active sprint" hoặc Chủ nhà explicit move từ "Next sprint" lên. Không phiếu cho item ở "Open backlog" / "Park" cho đến khi Chủ nhà pick.

---

## 🔥 Active sprint: <Sprint name / number>

> **Mục tiêu:** <1-2 câu mô tả mục tiêu sprint này.>
> **Kết thúc khi:** <Điều kiện kết thúc — không phải deadline thời gian.>
> **Started:** <DD/MM/YYYY khi pick lên Active>

<!-- Liệt kê 3-7 item Chủ nhà đã commit sẽ làm trong sprint này.
     Dùng tag để phân loại:
       [NEW]      = idea mới từ Chủ nhà
       [DEBT]     = tech debt từ Discovery hoặc retro
       [BUGFIX]   = bug Chủ nhà report
       [RESEARCH] = cần investigate trước
-->

- [ ] **[NEW]** <Item 1 — short summary>
- [ ] **[NEW]** <Item 2>

---

## 🎯 Next sprint: <Sprint name / theme>

> **Trigger:** <Khi nào move lên Active — vd "khi sprint hiện tại xong + Chủ nhà dùng feedback X".>
> **Theme:** <Một câu mô tả chủ đề sprint này.>

<!-- Idea cluster đã thành hình nhưng chưa active. Có thể thay đổi. -->

- [ ] <Item planned for next sprint>

---

## 🌊 Future waves (cam kết level low)

> Idea cluster lớn hơn — Phase / Sprint xa. Có thể thay đổi nhiều.

- [ ] **<Future Sprint name>** — <high-level description>
  - <sub-bullet>
  - <sub-bullet>

---

## 💡 Open backlog (chưa thuộc sprint)

> Idea rời, chưa cluster thành sprint. Khi đủ 2-3 cái cùng chủ đề → cluster.

<!-- Sếp dump idea ở đây qua /idea skill, hoặc trực tiếp edit tay. -->

- [ ] <Idea 1 — gì + tại sao + estimate sơ bộ>

---

## 🅿️ Park / nghĩ thêm

> Idea chưa chín, hoặc đã suy nghĩ nhưng chưa quyết, hoặc bị reject mềm (chưa hẳn no).

- [ ] <Idea cần research, hoặc đã debate chưa kết>

---

## ✅ Recently shipped (3 sprint gần nhất)

> Quick reference. Chi tiết đầy đủ → CHANGELOG.md.

<!-- Khi sprint xong, move tóm tắt 1-line vào đây. Giữ tối đa 3 sprint gần nhất. -->

- ✅ **<Sprint name>** (DD/MM/YYYY) — <one-line summary>

---

## ❌ Đã reject (lưu để khỏi nghĩ lại)

> Idea đã suy nghĩ và quyết KHÔNG làm. Lưu lý do rõ ràng để 6 tháng sau khỏi reconsider.

- **<Idea name>** — reject DD/MM/YYYY, lý do: <ngắn gọn>

---

## 📌 Quy tắc maintenance

1. **Idea mới** → `/idea` skill → tự append vào "Open backlog" hoặc "Active sprint" tùy phân loại.
2. **Phiếu xong** → move item từ Active sprint xuống "Recently shipped".
3. **Sprint xong** → tổng kết trong CHANGELOG.md, BACKLOG chỉ giữ 3 sprint gần nhất ở "Recently shipped".
4. **Discovery debt** mới → từ DISCOVERIES.md → append vào "Open backlog" với prefix `[DEBT]`.
5. **Architect rule** (cứng): không viết phiếu cho item nằm ngoài "Active sprint". Chủ nhà move item lên trước → Architect mới viết.
6. **Review monthly** — Chủ nhà đọc Park, quyết: promote lên Open backlog, hay ship to Reject với lý do.

---

*File này là LIVE. Chủ nhà chỉnh trực tiếp được. Architect/Worker chỉ ĐỌC, không tự edit khi đang viết phiếu.*
