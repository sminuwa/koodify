toastr.options = {
  closeButton: true,
  progressBar: true,
  positionClass: "toast-top-right",
  timeOut: 4000,
  newestOnTop: true,
  preventDuplicates: true
};

function showToast(type, message) {
  if (!message || typeof toastr === "undefined") return;
  toastr[type === "error" ? "error" : "success"](message);
}

function enhanceSelects(root) {
  if (typeof window.jQuery === "undefined" || !window.jQuery.fn.select2) return;
  const $ = window.jQuery;
  const scope = root && root.nodeType === 1 ? root : document;
  const selects = [];
  if (scope.matches && scope.matches("select")) selects.push(scope);
  if (scope.querySelectorAll) scope.querySelectorAll("select").forEach(function (select) { selects.push(select); });
  selects.forEach(function (select) {
    if (select.dataset.select2 === "off") return;
    const $select = $(select);
    if ($select.hasClass("select2-hidden-accessible")) return;
    const options = {
      theme: "bootstrap-5",
      width: select.dataset.width || "100%",
      minimumResultsForSearch: select.dataset.search === "always" || select.options.length > 8 ? 0 : Infinity,
      closeOnSelect: !select.multiple
    };
    const modal = select.closest(".zainbox-modal");
    if (modal) options.dropdownParent = $(document.body);
    if (select.dataset.placeholder) options.placeholder = select.dataset.placeholder;
    $select.select2(options);
    syncSelect2(select);
    $select.on("change.select2livewire", function () {
      if (select.dataset.select2Syncing === "1") return;
      select.dataset.select2Syncing = "1";
      select.dispatchEvent(new Event("change", { bubbles: true }));
      delete select.dataset.select2Syncing;
    });
  });
}

function syncSelect2(select) {
  if (typeof window.jQuery === "undefined" || !window.jQuery.fn.select2) return;
  const $select = window.jQuery(select);
  if (!$select.hasClass("select2-hidden-accessible")) return;
  const values = Array.from(select.selectedOptions).map(function (option) { return option.value; });
  select.dataset.select2Syncing = "1";
  $select.val(select.multiple ? values : (values[0] || "")).trigger("change.select2");
  delete select.dataset.select2Syncing;
}

function syncSelects(root) {
  const scope = root && root.nodeType === 1 ? root : document;
  const selects = [];
  if (scope.matches && scope.matches("select")) selects.push(scope);
  if (scope.querySelectorAll) scope.querySelectorAll("select").forEach(function (select) { selects.push(select); });
  selects.forEach(function (select) {
    if (select.dataset.select2 === "off") return;
    if (!window.jQuery(select).hasClass("select2-hidden-accessible")) {
      enhanceSelects(select);
      return;
    }
    syncSelect2(select);
  });
}

function releaseSelects(root) {
  if (!root || typeof window.jQuery === "undefined" || !window.jQuery.fn.select2) return;
  const $ = window.jQuery;
  const selects = [];
  if (root.tagName === "SELECT") selects.push(root);
  if (root.querySelectorAll) root.querySelectorAll("select").forEach(function (select) { selects.push(select); });
  selects.forEach(function (select) {
    const $select = $(select);
    if (!$select.hasClass("select2-hidden-accessible")) return;
    select.dataset.select2Syncing = "1";
    $select.off(".select2livewire");
    $select.select2("destroy");
    delete select.dataset.select2Syncing;
  });
}

document.addEventListener("livewire:init", function () {
  Livewire.on("toast", function (event) {
    const payload = Array.isArray(event) ? event[0] : event;
    showToast(payload.type, payload.message);
  });
  Livewire.hook("morph.updating", function ({ el }) {
    releaseSelects(el);
    releaseManagersTable(el);
  });
  Livewire.hook("morph.updated", function ({ el }) { enhanceSelects(el); });
  Livewire.hook("morph.added", function ({ el }) { enhanceSelects(el); });
  Livewire.hook("commit", function ({ succeed }) {
    succeed(function () {
      renderCollectedChart();
      renderDashboardCharts();
      paintManagersTable();
      syncSelects(document);
    });
  });
});

document.addEventListener("livewire:initialized", function () {
  renderCollectedChart();
  renderDashboardCharts();
  paintManagersTable();
});

function releaseManagersTable(root) {
  if (!root || typeof DataTable === "undefined" || !window.jQuery || !window.jQuery.fn.dataTable) return;
  const table = root.id === "managersTable" ? root : root.querySelector ? root.querySelector("#managersTable") : null;
  if (!table || !window.jQuery.fn.dataTable.isDataTable(table)) return;
  window.jQuery(table).DataTable().destroy();
}

function paintManagersTable() {
  const table = document.getElementById("managersTable");
  if (!table || typeof DataTable === "undefined" || !window.jQuery || !window.jQuery.fn.dataTable) return;
  if (window.jQuery.fn.dataTable.isDataTable(table)) return;
  new DataTable(table, {
    pageLength: 10,
    lengthChange: false,
    ordering: true,
    searching: false,
    autoWidth: false,
    columnDefs: [
      { orderable: false, targets: -1 },
      { className: "text-nowrap", targets: "_all" }
    ],
    layout: { topStart: null, topEnd: null, bottomStart: "info", bottomEnd: "paging" },
    language: {
      info: "Showing _START_ to _END_ of _TOTAL_ managers",
      infoEmpty: "Showing 0 to 0 of 0 managers",
      emptyTable: "No managers yet."
    }
  });
}

function placeMobileSidebar() {
  const shell = document.getElementById("appShell");
  const bar = shell ? shell.querySelector(".topbar") : null;

  if (!shell || !bar) {
    return;
  }

  shell.style.setProperty("--topbar-offset", Math.ceil(bar.getBoundingClientRect().height) + "px");
}

$(function () {
  placeMobileSidebar();
  $(window).on("resize", placeMobileSidebar);
  $("#collapseSidebar").on("click", function () { $("#appShell").toggleClass("collapsed"); });
  $("#mobileMenu").on("click", function () { $("#sidebar").toggleClass("mobile-open"); });
  $(document).on("click", function (e) {
    if (!$(e.target).closest("#sidebar,#mobileMenu").length) $("#sidebar").removeClass("mobile-open");
  });

  if (localStorage.getItem("zainpay-theme") === "dark") {
    $("body").addClass("dark-mode");
    $("#themeToggle i").removeClass("bi-moon").addClass("bi-sun");
  }
  $("#themeToggle").on("click", function () {
    $("body").toggleClass("dark-mode");
    $(this).find("i").toggleClass("bi-moon bi-sun");
    localStorage.setItem("zainpay-theme", $("body").hasClass("dark-mode") ? "dark" : "light");
    if ($("#flowChart").length) renderCharts();
    renderCollectedChart();
    renderDashboardCharts();
  });

  const flash = document.body.dataset.toast;
  if (flash) showToast("success", flash);

  $(document).on("click", "[data-confirm-transfer]", function (event) {
    event.preventDefault();
    const root = this.closest("[wire\\:id]");
    if (!root || typeof Sweetalert2 === "undefined") return;

    Sweetalert2.fire({
      title: "Send this transfer?",
      text: "The amount leaves the source account.",
      icon: "warning",
      showCancelButton: true,
      confirmButtonText: "Send transfer",
      cancelButtonText: "Cancel",
      reverseButtons: true,
      focusCancel: true,
      buttonsStyling: false,
      customClass: {
        popup: "transfer-swal",
        title: "transfer-swal-title",
        htmlContainer: "transfer-swal-text",
        actions: "transfer-swal-actions",
        confirmButton: "btn btn-primary",
        cancelButton: "btn btn-outline-primary"
      }
    }).then(function (result) {
      if (!result.isConfirmed) return;
      Livewire.find(root.getAttribute("wire:id")).send();
    });
  });

  $("#globalSearch").on("input", function () {
    const query = $(this).val().trim().toLowerCase();
    $(".side-nav .nav-link").each(function () {
      $(this).toggle(!query || $(this).text().toLowerCase().includes(query));
    });
  });
  $(document).on("keydown", function (e) {
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
      e.preventDefault();
      $("#globalSearch").trigger("focus");
    }
  });

  enhanceSelects(document);

  if ($("#envSelect").length) {
    const configuredEnv = $("#envSelect").data("env");
    if (configuredEnv) $("#envSelect").val(configuredEnv).trigger("change.select2");
    $("#envSelect").on("change", function () {
        const live = $(this).val() === "live";
        $(".user-env").text(live ? "Production" : "Sandbox");
        $(".key-mask").text(live ? "zp_live_••••4921" : "zp_test_••••4921");
        $(".key-note").text(live
          ? "Production key for this business. It is masked and not a live secret."
          : "Sandbox key for this business. It is masked and not a live secret.");
      });
  }

  renderCollectedChart();
  renderDashboardCharts();

  if (!$("#dashTable").length) return;

  const dashRows = [
    ["Today, 10:24", "Amaka Adeyemi", "ZP-20418", "Card", "₦125,000.00", "in", "Successful"],
    ["Today, 09:41", "GTBank transfer", "ZP-20411", "Bank transfer", "₦80,000.00", "in", "Successful"],
    ["Today, 08:41", "Chioma Nwosu", "ZP-20402", "Transfer", "₦64,000.00", "out", "Pending"],
    ["Yesterday, 18:12", "Kuda card", "ZP-20388", "Card", "₦18,500.00", "in", "Failed"],
    ["Yesterday, 14:03", "Ibrahim Musa", "ZP-20371", "Zainbox", "₦410,000.00", "in", "Successful"],
    ["Yesterday, 11:20", "Access Bank", "ZP-20360", "Bank transfer", "₦250,000.00", "in", "Successful"],
    ["11 Oct, 16:05", "Fatima Bello", "ZP-20340", "Card", "₦42,000.00", "in", "Successful"],
    ["11 Oct, 09:12", "Payroll", "ZP-20321", "Transfer", "₦1,200,000.00", "out", "Successful"]
  ];
  let dashTable = null;

  function paintDash() {
    const filter = $("#dashType").val() || "all";
    const rows = dashRows.filter(function (row) {
      if (filter === "in") return row[5] === "in";
      if (filter === "out") return row[5] === "out";
      if (filter === "failed") return row[6] === "Failed";
      return true;
    });
    if (dashTable) {
      dashTable.destroy();
      dashTable = null;
    }
    $("#dashTable").empty();
    dashTable = new DataTable("#dashTable", {
      data: rows,
      columns: [
        { title: "When", data: 0 },
        { title: "Customer", data: 1, render: function (value, type, row) {
          return "<b>" + value + "</b><small class=\"tx-ref\">" + row[2] + " · " + row[3] + "</small>";
        }},
        { title: "Amount", data: 4, render: function (value, type, row) {
          const sign = row[5] === "out" ? "−" : "+";
          const tone = row[6] === "Failed" ? "amount-bad" : row[5] === "out" ? "amount-out" : "amount-in";
          return "<span class=\"" + tone + "\">" + sign + value + "</span>";
        }},
        { title: "Status", data: 6, render: function (value) {
          const kind = value === "Successful" ? "is-ok" : value === "Failed" ? "is-bad" : "is-wait";
          return "<span class=\"status-pill " + kind + "\">" + value + "</span>";
        }}
      ],
      pageLength: 6,
      lengthChange: false,
      ordering: false,
      layout: { topStart: "search", topEnd: null, bottomStart: "info", bottomEnd: "paging" },
      language: { search: "", emptyTable: "No matching transactions.", zeroRecords: "No matching transactions." }
    });
    $("#opsDashboard .dt-search input").attr("placeholder", "Search transactions");
  }

  $("#dashType").on("change", paintDash);
  paintDash();
  renderCharts();
  renderCollectedChart();
});

let flowChart = null;
let channelChart = null;
let payoutChart = null;
let collectedChart = null;
let dashboardFlowChart = null;
let dashboardMixChart = null;
let dashboardBoxesChart = null;

function renderCharts() {
  if (typeof Chart === "undefined" || !document.getElementById("flowChart")) return;
  const dark = document.body.classList.contains("dark-mode");
  const muted = dark ? "#c5d2ea" : "#3c5278";
  const grid = dark ? "rgba(197, 210, 234, 0.16)" : "#e6edf7";
  const reduce = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  const days = ["7 Oct", "8 Oct", "9 Oct", "10 Oct", "11 Oct", "12 Oct", "13 Oct"];
  const money = function (value) { return "₦" + Number(value).toLocaleString("en-NG"); };
  const nairaTick = function (value) {
    if (value >= 1000000) return "₦" + (value / 1000000).toFixed(value % 1000000 ? 1 : 0) + "m";
    if (value >= 1000) return "₦" + Math.round(value / 1000) + "k";
    return "₦" + value;
  };
  Chart.defaults.font.family = '"Instrument Sans", "Segoe UI", sans-serif';
  Chart.defaults.color = muted;
  if (flowChart) flowChart.destroy();
  if (channelChart) channelChart.destroy();
  if (payoutChart) payoutChart.destroy();
  flowChart = new Chart(document.getElementById("flowChart"), {
    type: "line",
    data: {
      labels: days,
      datasets: [
        { label: "Money in", data: [920000, 1480000, 640000, 2100000, 1282000, 1800000, 4820000], borderColor: "#5b4dff", backgroundColor: "rgba(91, 77, 255, 0.12)", fill: true, tension: 0.35, pointRadius: 0, pointHoverRadius: 4, borderWidth: 2 },
        { label: "Money out", data: [180000, 240000, 90000, 310000, 1200000, 220000, 64000], borderColor: dark ? "#c5d4f5" : "#0b1b33", backgroundColor: dark ? "rgba(197, 212, 245, 0.12)" : "rgba(11, 27, 51, 0.08)", fill: true, tension: 0.35, pointRadius: 0, pointHoverRadius: 4, borderWidth: 2 }
      ]
    },
    options: {
      responsive: true,
      maintainAspectRatio: false,
      animation: reduce ? false : { duration: 500 },
      interaction: { mode: "index", intersect: false },
      plugins: {
        legend: { position: "bottom", labels: { boxWidth: 10, boxHeight: 10, usePointStyle: true } },
        tooltip: { callbacks: { label: function (item) { return item.dataset.label + ": " + money(item.parsed.y); } } }
      },
      scales: {
        x: { grid: { display: false } },
        y: { beginAtZero: true, grid: { color: grid }, ticks: { callback: nairaTick } }
      }
    }
  });
  channelChart = new Chart(document.getElementById("channelChart"), {
    type: "doughnut",
    data: {
      labels: ["Card", "Bank transfer", "Zainbox"],
      datasets: [{ data: [46, 38, 16], backgroundColor: dark ? ["#4c8dff", "#c5d4f5", "#3dbe7a"] : ["#5b4dff", "#0b1b33", "#0b7a45"], borderWidth: 0, hoverOffset: 4 }]
    },
    options: {
      responsive: true,
      maintainAspectRatio: false,
      cutout: "68%",
      animation: reduce ? false : { duration: 500 },
      plugins: {
        legend: { position: "bottom", labels: { boxWidth: 10, boxHeight: 10, usePointStyle: true } },
        tooltip: { callbacks: { label: function (item) { return item.label + ": " + item.parsed + "%"; } } }
      }
    }
  });
  payoutChart = new Chart(document.getElementById("payoutChart"), {
    type: "bar",
    data: {
      labels: days,
      datasets: [{ label: "Settled", data: [0, 0, 0, 0, 0, 850000, 2000000], backgroundColor: "#5b4dff", borderRadius: 6, maxBarThickness: 18 }]
    },
    options: {
      responsive: true,
      maintainAspectRatio: false,
      animation: reduce ? false : { duration: 500 },
      plugins: {
        legend: { display: false },
        tooltip: { callbacks: { label: function (item) { return "Settled: " + money(item.parsed.y); } } }
      },
      scales: {
        x: { grid: { display: false } },
        y: { beginAtZero: true, grid: { color: grid }, ticks: { callback: nairaTick } }
      }
    }
  });
}

function chartDayTick(scale, value, index) {
  const labels = scale.chart.data.labels || [];
  const last = labels.length - 1;
  if (last <= 15) return scale.getLabelForValue(value);
  const keep = { 0: true };
  keep[last] = true;
  for (let slot = 1; slot < 5; slot++) keep[Math.round(slot * last / 5)] = true;
  return keep[index] ? scale.getLabelForValue(value) : "";
}

function chartKey(canvas) {
  const theme = document.body.classList.contains("dark-mode") ? "dark" : "light";

  return theme + "\n" + (canvas.dataset.series || "");
}

function chartStillCurrent(chart, canvas) {
  return !!(chart && canvas && chart.canvas === canvas && chart.$key === chartKey(canvas));
}

function renderCollectedChart() {
  const canvas = document.getElementById("collectedChart");
  if (chartStillCurrent(collectedChart, canvas)) return;
  if (collectedChart) {
    collectedChart.destroy();
    collectedChart = null;
  }
  if (!canvas || typeof Chart === "undefined") return;
  let series = [];
  try {
    series = JSON.parse(canvas.dataset.series || "[]");
  } catch (error) {
    return;
  }
  if (!series.length) return;
  const dark = document.body.classList.contains("dark-mode");
  const muted = dark ? "#c5d2ea" : "#3c5278";
  const grid = dark ? "rgba(197, 210, 234, 0.16)" : "#e6edf7";
  const reduce = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  const money = function (value) {
    return "₦" + Number(value).toLocaleString("en-NG", { minimumFractionDigits: 2, maximumFractionDigits: 2 });
  };
  const nairaTick = function (value) {
    if (value >= 1000000) return "₦" + (value / 1000000).toFixed(value % 1000000 ? 1 : 0) + "m";
    if (value >= 1000) return "₦" + Math.round(value / 1000) + "k";
    return "₦" + value;
  };
  Chart.defaults.font.family = '"Instrument Sans", "Segoe UI", sans-serif';
  Chart.defaults.color = muted;
  const line = function (label, values, color, fill) {
    return {
      label: label,
      data: values,
      borderColor: color,
      backgroundColor: fill ? (dark ? "rgba(76, 141, 255, 0.16)" : "rgba(91, 77, 255, 0.12)") : "transparent",
      fill: fill,
      tension: 0.35,
      pointRadius: values.map(function (_, index) { return index === values.length - 1 ? 4 : 0; }),
      pointHoverRadius: 5,
      pointBackgroundColor: color,
      borderWidth: 2
    };
  };
  const hasFlow = series.some(function (point) { return point.in !== undefined || point.out !== undefined; });
  collectedChart = new Chart(canvas, {
    type: "line",
    data: {
      labels: series.map(function (point) { return point.label; }),
      datasets: hasFlow ? [
        line("Money in", series.map(function (point) { return point.in || 0; }), "#5b4dff", true),
        line("Money out", series.map(function (point) { return point.out || 0; }), "#b7c3d6", false)
      ] : [
        line("Collected", series.map(function (point) { return point.amount; }), "#5b4dff", true)
      ]
    },
    options: {
      responsive: true,
      maintainAspectRatio: false,
      animation: reduce ? false : { duration: 500 },
      interaction: { mode: "index", intersect: false },
      plugins: {
        legend: { display: false },
        tooltip: {
          callbacks: {
            title: function (items) {
              const point = series[items[0].dataIndex] || {};
              return point.full || items[0].label;
            },
            label: function (item) {
              const name = item.dataset.label ? item.dataset.label + " " : "";
              return name + money(item.parsed.y);
            }
          }
        }
      },
      scales: {
        x: {
          grid: { display: false },
          ticks: {
            autoSkip: false,
            maxRotation: 0,
            callback: function (value, index) { return chartDayTick(this, value, index); }
          }
        },
        y: { beginAtZero: true, grid: { color: grid }, ticks: { callback: nairaTick } }
      }
    }
  });
  collectedChart.$key = chartKey(canvas);
}

function paintSparks(series, drawnKey) {
  document.querySelectorAll("[data-spark]").forEach(function (el) {
    if (drawnKey && el.dataset.sparkDrawn === drawnKey) return;
    const key = el.dataset.spark;
    const color = el.dataset.sparkColor || "#5b4dff";
    const values = (series || []).map(function (point) {
      if (key === "both") return (point.in || 0) + (point.out || 0);
      return point[key] || 0;
    });
    el.dataset.sparkDrawn = drawnKey || "";
    el.innerHTML = sparkSvg(values, color);
  });
}

function sparkSvg(values, color) {
  const width = 112;
  const height = 46;
  if (!values.length) return "";
  const max = Math.max.apply(null, values.concat([0]));
  const span = max || 1;
  const step = values.length === 1 ? 0 : width / (values.length - 1);
  const coords = values.map(function (value, index) {
    return [index * step, height - 4 - (value / span) * (height - 10)];
  });
  const line = coords.map(function (point, index) {
    return (index === 0 ? "M" : "L") + point[0].toFixed(1) + " " + point[1].toFixed(1);
  }).join(" ");
  const area = line + " L" + width + " " + height + " L0 " + height + " Z";
  return '<svg viewBox="0 0 ' + width + ' ' + height + '" preserveAspectRatio="none">' +
    '<path d="' + area + '" fill="' + color + '" opacity="0.12"></path>' +
    '<path d="' + line + '" fill="none" stroke="' + color + '" stroke-width="2" stroke-linejoin="round" stroke-linecap="round"></path>' +
    '</svg>';
}

function renderDashboardCharts() {
  if (typeof Chart === "undefined") return;
  const dark = document.body.classList.contains("dark-mode");
  const muted = dark ? "#c5d2ea" : "#3c5278";
  const grid = dark ? "rgba(197, 210, 234, 0.16)" : "#e6edf7";
  const reduce = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  const money = function (value) {
    return "₦" + Number(value).toLocaleString("en-NG", { minimumFractionDigits: 2, maximumFractionDigits: 2 });
  };
  const nairaTick = function (value) {
    if (value >= 1000000) return "₦" + (value / 1000000).toFixed(value % 1000000 ? 1 : 0) + "m";
    if (value >= 1000) return "₦" + Math.round(value / 1000) + "k";
    return "₦" + value;
  };
  const readSeries = function (canvas) {
    try {
      return JSON.parse(canvas.dataset.series || "[]");
    } catch (error) {
      return [];
    }
  };
  Chart.defaults.font.family = '"Instrument Sans", "Segoe UI", sans-serif';
  Chart.defaults.color = muted;

  const flow = document.getElementById("dashboardFlow");
  if (chartStillCurrent(dashboardFlowChart, flow)) {
    paintSparks(readSeries(flow), chartKey(flow));
  } else {
    if (dashboardFlowChart) {
      dashboardFlowChart.destroy();
      dashboardFlowChart = null;
    }
    if (flow) {
      const series = readSeries(flow);
      if (series.length) {
        dashboardFlowChart = new Chart(flow, {
        type: "line",
        data: {
          labels: series.map(function (point) { return point.label; }),
          datasets: [
            {
              label: "Money In",
              data: series.map(function (point) { return point.in; }),
              borderColor: "#5b4dff",
              backgroundColor: "rgba(47, 107, 255, 0.08)",
              fill: true,
              tension: 0.32,
              pointRadius: 4,
              pointHoverRadius: 6,
              pointBackgroundColor: "#fff",
              pointBorderColor: "#5b4dff",
              pointBorderWidth: 2,
              borderWidth: 2
            },
            {
              label: "Money Out",
              data: series.map(function (point) { return point.out; }),
              borderColor: "#c5ceda",
              backgroundColor: "transparent",
              fill: false,
              tension: 0.32,
              pointRadius: 0,
              pointHoverRadius: 4,
              borderWidth: 2
            }
          ]
        },
        options: {
          responsive: true,
          maintainAspectRatio: false,
          animation: reduce ? false : { duration: 500 },
          interaction: { mode: "index", intersect: false },
          plugins: {
            legend: { display: false },
            tooltip: { callbacks: { label: function (item) { return item.dataset.label + ": " + money(item.parsed.y); } } }
          },
          scales: {
            x: {
              grid: { display: false },
              border: { display: false },
              ticks: {
                color: muted,
                autoSkip: true,
                maxRotation: 0,
                maxTicksLimit: 8
              }
            },
            y: {
              beginAtZero: true,
              border: { display: false },
              grid: { color: grid, drawTicks: false },
              ticks: { color: muted, padding: 8, callback: nairaTick }
            }
          }
        }
      });
        dashboardFlowChart.$key = chartKey(flow);
      }
      paintSparks(series, chartKey(flow));
    }
  }

  const mix = document.getElementById("dashboardMix");
  if (!chartStillCurrent(dashboardMixChart, mix) && dashboardMixChart) {
    dashboardMixChart.destroy();
    dashboardMixChart = null;
  }
  if (mix && !dashboardMixChart) {
    const series = readSeries(mix);
    if (series.length) {
      dashboardMixChart = new Chart(mix, {
        type: "doughnut",
        data: {
          labels: series.map(function (point) { return point.label; }),
          datasets: [{ data: series.map(function (point) { return point.amount; }), backgroundColor: dark ? ["#4c8dff", "#c5d4f5", "#3dbe7a", "#f0b429"] : ["#5b4dff", "#0b1b33", "#0b7a45", "#c47b12"], borderWidth: 0, hoverOffset: 4 }]
        },
        options: {
          responsive: true,
          maintainAspectRatio: false,
          cutout: "68%",
          animation: reduce ? false : { duration: 500 },
          plugins: {
            legend: { position: "bottom", labels: { boxWidth: 10, boxHeight: 10, usePointStyle: true } },
            tooltip: { callbacks: { label: function (item) { return item.label + ": " + money(item.parsed); } } }
          }
        }
      });
      dashboardMixChart.$key = chartKey(mix);
    }
  }

  const boxes = document.getElementById("dashboardBoxes");
  if (!chartStillCurrent(dashboardBoxesChart, boxes) && dashboardBoxesChart) {
    dashboardBoxesChart.destroy();
    dashboardBoxesChart = null;
  }
  if (boxes && !dashboardBoxesChart) {
    const series = readSeries(boxes);
    if (series.length) {
      dashboardBoxesChart = new Chart(boxes, {
        type: "bar",
        data: {
          labels: series.map(function (point) { return point.label; }),
          datasets: [{ label: "Collected", data: series.map(function (point) { return point.amount; }), backgroundColor: "#5b4dff", borderRadius: 6, maxBarThickness: 28 }]
        },
        options: {
          responsive: true,
          maintainAspectRatio: false,
          animation: reduce ? false : { duration: 500 },
          plugins: {
            legend: { display: false },
            tooltip: { callbacks: { label: function (item) { return "Collected: " + money(item.parsed.y); } } }
          },
          scales: {
            x: { grid: { display: false } },
            y: { beginAtZero: true, grid: { color: grid }, ticks: { callback: nairaTick } }
          }
        }
      });
      dashboardBoxesChart.$key = chartKey(boxes);
    }
  }
}
