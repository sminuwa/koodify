(() => {
  const $ = (s, root = document) => root.querySelector(s);
  const $$ = (s, root = document) => [...root.querySelectorAll(s)];
  const toast = $("#toast");

  function notify(message) {
    if (!toast) return;
    toast.textContent = message;
    toast.classList.add("show");
    clearTimeout(notify.timer);
    notify.timer = setTimeout(() => toast.classList.remove("show"), 2800);
  }

  function drawChart(mode = "volume") {
    const canvas = $("#volumeChart");
    if (!canvas) return;
    const rect = canvas.getBoundingClientRect();
    const dpr = window.devicePixelRatio || 1;
    canvas.width = Math.max(300, rect.width * dpr);
    canvas.height = 174 * dpr;
    const ctx = canvas.getContext("2d");
    ctx.setTransform(1, 0, 0, 1, 0, 0);
    ctx.scale(dpr, dpr);
    const w = canvas.width / dpr;
    const h = 174;
    const vals =
      mode === "count"
        ? [32, 48, 42, 65, 58, 86, 74, 100, 89, 112, 104, 132]
        : [52, 70, 64, 95, 82, 120, 112, 155, 143, 185, 170, 218];
    const max = mode === "count" ? 150 : 250;
    const x = (i) => (i / (vals.length - 1)) * (w - 8) + 4;
    const y = (v) => h - 8 - (v / max) * (h - 18);
    ctx.strokeStyle = "#eff0f6";
    ctx.lineWidth = 1;
    for (let i = 0; i < 6; i++) {
      const yy = 8 + i * 30;
      ctx.beginPath();
      ctx.moveTo(0, yy);
      ctx.lineTo(w, yy);
      ctx.stroke();
    }
    const points = vals.map((v, i) => [x(i), y(v)]);
    ctx.beginPath();
    points.forEach(([px, py], i) => (i ? ctx.lineTo(px, py) : ctx.moveTo(px, py)));
    ctx.lineTo(w, h);
    ctx.lineTo(0, h);
    ctx.closePath();
    const grad = ctx.createLinearGradient(0, 0, 0, h);
    grad.addColorStop(0, "rgba(117,80,238,.22)");
    grad.addColorStop(1, "rgba(117,80,238,0)");
    ctx.fillStyle = grad;
    ctx.fill();
    ctx.beginPath();
    points.forEach(([px, py], i) => (i ? ctx.lineTo(px, py) : ctx.moveTo(px, py)));
    ctx.strokeStyle = "#7955ef";
    ctx.lineWidth = 2.5;
    ctx.lineJoin = "round";
    ctx.lineCap = "round";
    ctx.stroke();
    points.forEach(([px, py], i) => {
      if (i === points.length - 1) {
        ctx.beginPath();
        ctx.arc(px, py, 3.5, 0, Math.PI * 2);
        ctx.fillStyle = "#7955ef";
        ctx.fill();
      }
    });
  }

  $$(".segmented button").forEach((btn) =>
    btn.addEventListener("click", () => {
      $$(".segmented button").forEach((x) => x.classList.remove("selected"));
      btn.classList.add("selected");
      const total = $("#chartTotal");
      if (total) {
        total.textContent =
          btn.dataset.chart === "volume" ? "₦2,457,930,000" : "1,248,320 transactions";
      }
      drawChart(btn.dataset.chart);
    })
  );

  $("#exportBtn")?.addEventListener("click", () => {
    const csv =
      "Reference,Merchant,Service,Date,Amount,Status\nTXN-839201,Swift Basket,Payment,Nov 07 2026,245000,Successful\nTXN-839200,Nova Goods,Transfer,Nov 07 2026,82500,Pending\nTXN-839199,Amina Mart,Virtual account,Nov 06 2026,1250000,Successful\nTXN-839198,FreshCart,Payment,Nov 06 2026,36700,Failed";
    const blob = new Blob([csv], { type: "text/csv" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = "koodify-transactions.csv";
    a.click();
    URL.revokeObjectURL(url);
    notify("Report exported.");
  });

  drawChart();
  window.addEventListener("resize", () =>
    drawChart($(".segmented button.selected")?.dataset.chart || "volume")
  );
})();
