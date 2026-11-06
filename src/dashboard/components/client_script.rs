pub fn get_client_javascript() -> &'static str {
    r#"
let payload = null;
let chartCurvas = null;
let chartDrawdowns = null;
let chartAsignacion = null;
let chartH2H = null;
let h2hSelectedIdA = null;
let h2hSelectedIdB = null;
let h2hSelectedIdC = 'none';
let currentEscala = 'log';
let currentRango = 'todo';
let currentDdm = 'pico'; // por defecto contra el capital total en ese momento
let selectedStratId = null;

document.addEventListener('DOMContentLoaded', () => {
  const dataEl = document.getElementById('datos');
  if (!dataEl) return;
  payload = JSON.parse(dataEl.textContent);

  // Seleccionar la primera estrategia por defecto
  if (payload.strategies && payload.strategies.length > 0) {
    selectedStratId = payload.strategies[0].id;
  }

  initNavigation();
  initLeaderboard();
  populateStrategySelectors();
  initHeadToHead();
  initCharts();
  renderYearlyTable();
  renderDetailKPIs();
  renderTradesTable();
  updateClock();
  setInterval(updateClock, 1000);
});

function updateClock() {
  const el = document.getElementById('reloj');
  if (!el) return;
  const now = new Date();
  el.innerHTML = 'UTC: <b>' + now.toUTCString().slice(17, 25) + '</b>';
}

function initNavigation() {
  // Tabs de Gráficos
  document.querySelectorAll('.tab-btn').forEach(btn => {
    btn.addEventListener('click', () => {
      document.querySelectorAll('.tab-btn').forEach(b => b.classList.remove('active'));
      document.querySelectorAll('.tab-panel').forEach(p => p.classList.remove('active'));
      btn.classList.add('active');
      const targetId = btn.getAttribute('data-tab');
      const targetPanel = document.getElementById(targetId);
      if (targetPanel) targetPanel.classList.add('active');

      if (targetId === 'tab-drawdowns' && chartDrawdowns) chartDrawdowns.resize();
      if (targetId === 'tab-asignacion' && chartAsignacion) {
        renderAsignacionChart();
        chartAsignacion.resize();
      }
      if (targetId === 'tab-curvas' && chartCurvas) chartCurvas.resize();
    });
  });

  // Seg Escala (Log / Lineal)
  document.querySelectorAll('#seg-escala button').forEach(b => {
    b.addEventListener('click', () => {
      document.querySelectorAll('#seg-escala button').forEach(btn => btn.classList.remove('active'));
      b.classList.add('active');
      currentEscala = b.getAttribute('data-val');
      if (chartCurvas) {
        chartCurvas.options.scales.y.type = currentEscala === 'log' ? 'logarithmic' : 'linear';
        chartCurvas.update();
      }
    });
  });

  // Seg Rango (Todo / IS / OOS)
  document.querySelectorAll('#seg-rango button').forEach(b => {
    b.addEventListener('click', () => {
      document.querySelectorAll('#seg-rango button').forEach(btn => btn.classList.remove('active'));
      b.classList.add('active');
      currentRango = b.getAttribute('data-val');
      updateChartsRange();
    });
  });

  // Control de Modo Drawdown (vs capital inicial / vs pico)
  document.querySelectorAll('#d-ddm button').forEach(b => {
    b.addEventListener('click', () => {
      document.querySelectorAll('#d-ddm button').forEach(btn => btn.classList.remove('active', 'on'));
      b.classList.add('active', 'on');
      currentDdm = b.getAttribute('data-v');
      const desc = document.getElementById('d-ddm-desc');
      if (desc) {
        desc.textContent = currentDdm === 'cap'
          ? 'Max DD calculado respecto al capital inicial de la cuenta ($10,000).'
          : 'Max DD calculado respecto al capital total alcanzado en ese momento (pico previo).';
      }
      renderDetailKPIs();
    });
  });

  // Select Estrategia Asignación
  const selectAlloc = document.getElementById('select-strat-allocation');
  if (selectAlloc) {
    selectAlloc.addEventListener('change', (e) => {
      renderAsignacionChart(e.target.value);
    });
  }

  // Select Estrategia Detalle
  const selectDetail = document.getElementById('select-strat-detail');
  if (selectDetail) {
    selectDetail.addEventListener('change', (e) => {
      selectStrategy(e.target.value);
    });
  }
}

function selectStrategy(id) {
  selectedStratId = id;
  const sd = document.getElementById('select-strat-detail');
  if (sd) sd.value = id;
  const sa = document.getElementById('select-strat-allocation');
  if (sa) sa.value = id;
  renderDetailKPIs();
  renderTradesTable();
  renderAsignacionChart(id);
  const detEl = document.getElementById('detalle');
  if (detEl) detEl.scrollIntoView({ behavior: 'smooth' });
}

function initLeaderboard() {
  // 1. Podio Top 3
  const podioEl = document.getElementById('lb-podio');
  if (podioEl && payload.strategies && payload.strategies.length > 0) {
    const medallas = ['🥇 1º', '🥈 2º', '🥉 3º'];
    podioEl.innerHTML = payload.strategies.slice(0, 3).map((s, k) => {
      const ch = s.section_metrics;
      const oosRet = ch ? ch.oos_section.return_pct : s.metrics.total_return_pct;
      const oosDd = ch ? (ch.oos_section.max_dd_pico_pct || ch.oos_section.max_dd_pct) : s.metrics.max_drawdown_pct;
      const scoreComp = ch ? ch.score_completo : s.metrics.swap_fitness_score;
      const retSign = oosRet >= 0 ? '+' : '';
      const retClass = oosRet >= 0 ? 'pos' : 'neg';

      return `
        <div class="pod p${k + 1}">
          <div class="cc-h">
            <span class="rk r${k + 1}">${medallas[k]}</span>
            <span class="pill gris">Score COMP ${scoreComp.toFixed(2)}</span>
          </div>
          <h3>
            <button class="lnk" type="button" onclick="selectStrategy('${s.id}')">
              <span class="dot" style="background:${s.color}"></span>
              <b>${s.name}</b>
            </button>
          </h3>
          <div class="pm">
            <div>
              <span class="l">Score OOS</span>
              <span class="v">${ch ? ch.score_oos.toFixed(2) : '-'}</span>
            </div>
            <div>
              <span class="l">Retorno OOS</span>
              <span class="v ${retClass}">${retSign}${oosRet.toFixed(2)}%</span>
            </div>
            <div>
              <span class="l">Max DD OOS (vs Cap Total)</span>
              <span class="v neg">-${oosDd.toFixed(2)}%</span>
            </div>
            <div>
              <span class="l">Ret · por velas</span>
              <span class="v">${ch ? ch.retention_pct.toFixed(0) + '% · ' + ch.retention_velas_pct.toFixed(0) + '%' : '-'}</span>
            </div>
          </div>
        </div>
      `;
    }).join('');
  }

  // 2. Tabla Leaderboard con Métricas Robustas
  const tbody = document.getElementById('leaderboard-body');
  if (!tbody) return;
  tbody.innerHTML = '';

  payload.strategies.forEach((s, idx) => {
    const tr = document.createElement('tr');
    tr.style.cursor = 'pointer';
    tr.onclick = () => selectStrategy(s.id);

    const ch = s.section_metrics;
    const medalla = idx === 0 ? '🥇' : idx === 1 ? '🥈' : idx === 2 ? '🥉' : `${idx + 1}`;
    const retClass = s.metrics.total_return_pct >= 0 ? 'val-pos' : 'val-neg';
    const retSign = s.metrics.total_return_pct >= 0 ? '+' : '';
    const oosRet = ch ? ch.oos_section.return_pct : 0.0;
    const oosRetClass = oosRet >= 0 ? 'val-pos' : 'val-neg';
    const oosRetSign = oosRet >= 0 ? '+' : '';
    const diffBh = ch ? ch.completo_section.diff_vs_bh_pct : 0.0;
    const diffBhClass = diffBh >= 0 ? 'val-pos' : 'val-neg';
    const diffBhSign = diffBh >= 0 ? '+' : '';

    tr.innerHTML = `
      <td><span class="rk ${idx < 3 ? 'r' + (idx + 1) : ''}">${medalla}</span></td>
      <td>
        <span class="dot" style="background:${s.color}"></span>
        <b>${s.name}</b>
        <div class="muted" style="font-size:10px;">${s.architecture} · <span style="color:var(--verde); font-weight:600;">${s.margin_mode || 'Capital Cruzado'} (${s.leverage ? s.leverage + 'X' : '5X'})</span></div>
      </td>
      <td class="al-r"><b>${ch ? ch.score_is.toFixed(2) : '-'}</b></td>
      <td class="al-r"><b>${ch ? ch.score_oos.toFixed(2) : '-'}</b></td>
      <td class="al-r"><b style="color:var(--sp500);">${ch ? ch.score_completo.toFixed(2) : s.metrics.swap_fitness_score.toFixed(2)}</b></td>
      <td class="al-r">${ch ? ch.retention_pct.toFixed(0) + '%' : '-'}</td>
      <td class="al-r">${ch ? ch.retention_velas_pct.toFixed(0) + '%' : '-'}</td>
      <td class="al-r ${retClass}"><b>${retSign}${s.metrics.total_return_pct.toFixed(2)}%</b><div class="muted" style="font-size:10px;">PnL $${s.net_profit.toFixed(0)}</div></td>
      <td class="al-r ${oosRetClass}"><b>${oosRetSign}${oosRet.toFixed(2)}%</b></td>
      <td class="al-r ${diffBhClass}"><b>${diffBhSign}${diffBh.toFixed(1)} pp</b></td>
      <td class="al-r val-neg">-${(ch ? (ch.oos_section.max_dd_pico_pct || ch.oos_section.max_dd_pct) : s.metrics.max_drawdown_pct).toFixed(2)}%</td>
      <td class="al-r gold">${ch ? ch.completo_section.max_stagnation_bars : s.metrics.max_drawdown_bars} v</td>
      <td class="al-r">${s.metrics.total_swaps}</td>
      <td class="al-r">${s.metrics.sharpe_ratio.toFixed(2)}</td>
      <td class="al-c">
        <span class="badge-dot dot-btc">BTC ${s.metrics.exposure_btc_pct.toFixed(0)}%</span>
        <span class="badge-dot dot-gold">ORO ${s.metrics.exposure_gold_pct.toFixed(0)}%</span>
        <span class="badge-dot dot-sp500">SPY ${s.metrics.exposure_sp500_pct.toFixed(0)}%</span>
        ${s.metrics.exposure_short_btc_pct > 0 ? `<span class="badge-dot dot-sh-btc">SH_BTC ${s.metrics.exposure_short_btc_pct.toFixed(0)}%</span>` : ''}
        ${s.metrics.exposure_short_gold_pct > 0 ? `<span class="badge-dot dot-sh-gold">SH_GOLD ${s.metrics.exposure_short_gold_pct.toFixed(0)}%</span>` : ''}
        ${s.metrics.exposure_short_sp500_pct > 0 ? `<span class="badge-dot dot-sh-sp500">SH_SPY ${s.metrics.exposure_short_sp500_pct.toFixed(0)}%</span>` : ''}
        <span class="badge-dot dot-cash">CASH ${s.metrics.exposure_cash_pct.toFixed(0)}%</span>
      </td>
    `;
    tbody.appendChild(tr);
  });

  // Separador de benchmarks
  const sepTr = document.createElement('tr');
  sepTr.innerHTML = `<td colspan="15" style="background:rgba(255,255,255,0.02); font-weight:700; color:var(--dim); font-size:10.5px; text-transform:uppercase; letter-spacing:0.08em; padding:6px 12px;">── Benchmarks Pasivos de Referencia (Buy & Hold) ──</td>`;
  tbody.appendChild(sepTr);

  // Benchmarks
  payload.benchmarks.forEach(b => {
    const tr = document.createElement('tr');
    const retClass = b.total_return_pct >= 0 ? 'val-pos' : 'val-neg';
    const retSign = b.total_return_pct >= 0 ? '+' : '';

    tr.innerHTML = `
      <td>-</td>
      <td><b style="color:${b.color};">●</b> ${b.name} <div class="muted" style="font-size:10px;">Benchmark Pasivo</div></td>
      <td class="al-r">-</td>
      <td class="al-r">-</td>
      <td class="al-r">-</td>
      <td class="al-r">-</td>
      <td class="al-r">-</td>
      <td class="al-r ${retClass}"><b>${retSign}${b.total_return_pct.toFixed(2)}%</b></td>
      <td class="al-r">-</td>
      <td class="al-r">-</td>
      <td class="al-r val-neg">-${b.max_drawdown_pct.toFixed(2)}%</td>
      <td class="al-r">-</td>
      <td class="al-r">0</td>
      <td class="al-r"><b>${b.sharpe_ratio.toFixed(2)}</b></td>
      <td class="al-c"><span class="muted">100% en ${b.symbol}</span></td>
    `;
    tbody.appendChild(tr);
  });
}

function populateStrategySelectors() {
  const sa = document.getElementById('select-strat-allocation');
  const sd = document.getElementById('select-strat-detail');
  if (!sa || !sd) return;

  sa.innerHTML = '';
  sd.innerHTML = '';

  payload.strategies.forEach(s => {
    const optA = document.createElement('option');
    optA.value = s.id;
    optA.textContent = s.name;
    sa.appendChild(optA);

    const optD = document.createElement('option');
    optD.value = s.id;
    optD.textContent = s.name;
    sd.appendChild(optD);
  });

  if (selectedStratId) {
    sa.value = selectedStratId;
    sd.value = selectedStratId;
  }
}

function getRangeIndices() {
  const total = payload.dates.length;
  if (currentRango === 'is') {
    return [0, payload.is_split_idx];
  } else if (currentRango === 'oos') {
    return [payload.is_split_idx, total];
  }
  return [0, total];
}

function initCharts() {
  const [startIdx, endIdx] = getRangeIndices();
  const visibleDates = payload.dates.slice(startIdx, endIdx);

  // 1. Configurar C1: Curvas Base 100 (Solo las 5 mejores estrategias)
  const ctxCurvas = document.getElementById('chart-curvas').getContext('2d');
  const datasetsCurvas = [];
  const topStrats = payload.strategies.slice(0, 5);

  topStrats.forEach(s => {
    const rawEq = s.equity_curve.slice(startIdx, endIdx);
    const baseVal = rawEq[0] || 1.0;
    const base100 = rawEq.map(v => (v / baseVal) * 100.0);

    datasetsCurvas.push({
      label: s.name,
      id: s.id,
      borderColor: s.color,
      backgroundColor: s.color,
      borderWidth: 2.2,
      pointRadius: 0,
      data: base100,
      tension: 0.1,
    });
  });

  payload.benchmarks.forEach(b => {
    const rawEq = b.equity_curve.slice(startIdx, endIdx);
    const baseVal = rawEq[0] || 1.0;
    const base100 = rawEq.map(v => (v / baseVal) * 100.0);

    datasetsCurvas.push({
      label: b.name,
      id: b.id,
      borderColor: b.color,
      backgroundColor: b.color,
      borderWidth: b.id === 'equal_weight' ? 2 : 1.5,
      borderDash: b.id.includes('btc') ? [4, 4] : b.id.includes('gold') ? [2, 2] : [6, 2],
      pointRadius: 0,
      data: base100,
      tension: 0.1,
    });
  });

  chartCurvas = new Chart(ctxCurvas, {
    type: 'line',
    data: {
      labels: visibleDates,
      datasets: datasetsCurvas,
    },
    options: {
      responsive: true,
      maintainAspectRatio: false,
      interaction: { mode: 'index', intersect: false },
      plugins: {
        legend: {
          display: true,
          position: 'top',
          labels: { color: '#9FAAB5', boxWidth: 14, font: { family: 'Montserrat', size: 11, weight: 'bold' } }
        },
        tooltip: {
          backgroundColor: 'rgba(15, 18, 22, 0.95)',
          titleColor: '#FFFFFF',
          bodyColor: '#EBE6E1',
          borderColor: 'rgba(255,255,255,0.1)',
          borderWidth: 1,
          padding: 10,
          callbacks: {
            label: function(context) {
              return `${context.dataset.label}: ${context.parsed.y.toFixed(2)} (Base 100)`;
            }
          }
        }
      },
      scales: {
        x: {
          grid: { color: 'rgba(255,255,255,0.05)' },
          ticks: { color: '#6C7885', maxTicksLimit: 12, font: { family: 'JetBrains Mono', size: 10 } }
        },
        y: {
          type: currentEscala === 'log' ? 'logarithmic' : 'linear',
          grid: { color: 'rgba(255,255,255,0.05)' },
          ticks: {
            color: '#9FAAB5',
            font: { family: 'JetBrains Mono', size: 11 },
            callback: (val) => val.toFixed(0)
          }
        }
      }
    }
  });

  // 2. Configurar C2: Drawdowns (Solo las 5 mejores estrategias vs benchmarks)
  const ctxDd = document.getElementById('chart-drawdowns').getContext('2d');
  const datasetsDd = [];

  topStrats.forEach(s => {
    datasetsDd.push({
      label: s.name,
      borderColor: s.color,
      backgroundColor: s.color,
      borderWidth: 1.8,
      pointRadius: 0,
      data: s.drawdown_curve.slice(startIdx, endIdx),
      tension: 0.1,
    });
  });

  payload.benchmarks.forEach(b => {
    datasetsDd.push({
      label: b.name,
      borderColor: b.color,
      backgroundColor: b.color,
      borderWidth: 1.2,
      borderDash: [4, 4],
      pointRadius: 0,
      data: b.drawdown_curve.slice(startIdx, endIdx),
      tension: 0.1,
    });
  });

  chartDrawdowns = new Chart(ctxDd, {
    type: 'line',
    data: {
      labels: visibleDates,
      datasets: datasetsDd,
    },
    options: {
      responsive: true,
      maintainAspectRatio: false,
      interaction: { mode: 'index', intersect: false },
      plugins: {
        legend: {
          display: true,
          position: 'top',
          labels: { color: '#9FAAB5', boxWidth: 14, font: { family: 'Montserrat', size: 11, weight: 'bold' } }
        },
        tooltip: {
          backgroundColor: 'rgba(15, 18, 22, 0.95)',
          titleColor: '#FFFFFF',
          bodyColor: '#EBE6E1',
          callbacks: {
            label: (ctx) => `${ctx.dataset.label}: -${ctx.parsed.y.toFixed(2)}%`
          }
        }
      },
      scales: {
        x: {
          grid: { color: 'rgba(255,255,255,0.05)' },
          ticks: { color: '#6C7885', maxTicksLimit: 12, font: { family: 'JetBrains Mono', size: 10 } }
        },
        y: {
          reverse: true,
          grid: { color: 'rgba(255,255,255,0.05)' },
          ticks: {
            color: '#FF2E4D',
            font: { family: 'JetBrains Mono', size: 11 },
            callback: (val) => `-${val}%`
          }
        }
      }
    }
  });

  // 3. Renderizar Asignación inicial
  renderAsignacionChart();
}

function updateChartsRange() {
  const [startIdx, endIdx] = getRangeIndices();
  const visibleDates = payload.dates.slice(startIdx, endIdx);

  // Actualizar C1
  if (chartCurvas) {
    chartCurvas.data.labels = visibleDates;
    chartCurvas.data.datasets.forEach(ds => {
      let fullEq = null;
      const strat = payload.strategies.find(s => s.name === ds.label);
      if (strat) fullEq = strat.equity_curve;
      const bm = payload.benchmarks.find(b => b.name === ds.label);
      if (bm) fullEq = bm.equity_curve;

      if (fullEq) {
        const rawEq = fullEq.slice(startIdx, endIdx);
        const baseVal = rawEq[0] || 1.0;
        ds.data = rawEq.map(v => (v / baseVal) * 100.0);
      }
    });
    chartCurvas.update();
  }

  // Actualizar C2
  if (chartDrawdowns) {
    chartDrawdowns.data.labels = visibleDates;
    chartDrawdowns.data.datasets.forEach(ds => {
      let fullDd = null;
      const strat = payload.strategies.find(s => s.name === ds.label);
      if (strat) fullDd = strat.drawdown_curve;
      const bm = payload.benchmarks.find(b => b.name === ds.label);
      if (bm) fullDd = bm.drawdown_curve;

      if (fullDd) {
        ds.data = fullDd.slice(startIdx, endIdx);
      }
    });
    chartDrawdowns.update();
  }

  renderAsignacionChart();
}

function renderAsignacionChart(stratId) {
  const targetId = stratId || selectedStratId || (payload.strategies[0] ? payload.strategies[0].id : null);
  const strat = payload.strategies.find(s => s.id === targetId);
  if (!strat) return;

  const [startIdx, endIdx] = getRangeIndices();
  const visibleDates = payload.dates.slice(startIdx, endIdx);
  const visibleAllocs = strat.allocations.slice(startIdx, endIdx);

  const seriesBtc = visibleAllocs.map(a => a.weight_btc !== undefined && a.weight_btc > 0 ? a.weight_btc : (a.asset === 'BTC' ? 1 : 0));
  const seriesGold = visibleAllocs.map(a => a.weight_gold !== undefined && a.weight_gold > 0 ? a.weight_gold : (a.asset === 'GOLD' ? 1 : 0));
  const seriesSp500 = visibleAllocs.map(a => a.weight_sp500 !== undefined && a.weight_sp500 > 0 ? a.weight_sp500 : (a.asset === 'SP500' ? 1 : 0));
  const seriesShBtc = visibleAllocs.map(a => a.weight_sh_btc !== undefined && a.weight_sh_btc > 0 ? a.weight_sh_btc : (a.asset === 'SH_BTC' ? 1 : 0));
  const seriesShGold = visibleAllocs.map(a => a.weight_sh_gold !== undefined && a.weight_sh_gold > 0 ? a.weight_sh_gold : (a.asset === 'SH_GOLD' ? 1 : 0));
  const seriesShSp500 = visibleAllocs.map(a => a.weight_sh_sp500 !== undefined && a.weight_sh_sp500 > 0 ? a.weight_sh_sp500 : (a.asset === 'SH_SP500' ? 1 : 0));
  const seriesCash = visibleAllocs.map(a => a.weight_cash !== undefined && a.weight_cash > 0 ? a.weight_cash : (a.asset === 'CASH' ? 1 : 0));

  const ctxAsig = document.getElementById('chart-asignacion').getContext('2d');
  if (chartAsignacion) {
    chartAsignacion.destroy();
  }

  chartAsignacion = new Chart(ctxAsig, {
    type: 'bar',
    data: {
      labels: visibleDates,
      datasets: [
        { label: 'Bitcoin (BTC)', data: seriesBtc, backgroundColor: '#F7931A', barPercentage: 1.0, categoryPercentage: 1.0 },
        { label: 'Oro Físico (PAXG)', data: seriesGold, backgroundColor: '#FFD700', barPercentage: 1.0, categoryPercentage: 1.0 },
        { label: 'S&P 500 (SPY)', data: seriesSp500, backgroundColor: '#00B0FF', barPercentage: 1.0, categoryPercentage: 1.0 },
        { label: 'Short BTC (SH_BTC)', data: seriesShBtc, backgroundColor: '#EF4444', barPercentage: 1.0, categoryPercentage: 1.0 },
        { label: 'Short Oro (SH_GOLD)', data: seriesShGold, backgroundColor: '#F59E0B', barPercentage: 1.0, categoryPercentage: 1.0 },
        { label: 'Short SP500 (SH_SP500)', data: seriesShSp500, backgroundColor: '#F43F5E', barPercentage: 1.0, categoryPercentage: 1.0 },
        { label: 'Dólar (CASH)', data: seriesCash, backgroundColor: '#10B981', barPercentage: 1.0, categoryPercentage: 1.0 },
      ]
    },
    options: {
      responsive: true,
      maintainAspectRatio: false,
      scales: {
        x: {
          stacked: true,
          grid: { display: false },
          ticks: { color: '#6C7885', maxTicksLimit: 12, font: { family: 'JetBrains Mono', size: 10 } }
        },
        y: {
          stacked: true,
          max: 1,
          grid: { color: 'rgba(255,255,255,0.05)' },
          ticks: {
            color: '#9FAAB5',
            font: { family: 'JetBrains Mono', size: 10 },
            callback: (v) => v === 1 ? '100%' : (v === 0.5 ? '50%' : (v === 0 ? '0%' : ''))
          }
        }
      },
      plugins: {
        legend: { display: true, position: 'top', labels: { color: '#9FAAB5', font: { family: 'Montserrat', size: 11 } } },
        tooltip: {
          callbacks: {
            title: (items) => items[0].label,
            label: (item) => item.raw > 0 ? `Posición: ${item.dataset.label} (${Math.round(item.raw * 100)}%)` : null
          }
        }
      }
    }
  });
}

function renderYearlyTable() {
  const tbody = document.getElementById('yearly-matrix-body');
  if (!tbody) return;
  tbody.innerHTML = '';

  payload.strategies.forEach(s => {
    s.yearly_returns.forEach(yr => {
      const tr = document.createElement('tr');
      const stratClass = yr.strategy_return_pct >= 0 ? 'val-pos' : 'val-neg';
      const btcClass = yr.btc_return_pct >= 0 ? 'val-pos' : 'val-neg';
      const goldClass = yr.gold_return_pct >= 0 ? 'val-pos' : 'val-neg';
      const spClass = yr.sp500_return_pct >= 0 ? 'val-pos' : 'val-neg';
      const alphaBtcClass = yr.diff_vs_btc >= 0 ? 'val-pos' : 'val-neg';
      const alphaSpClass = yr.diff_vs_sp500 >= 0 ? 'val-pos' : 'val-neg';

      tr.innerHTML = `
        <td><b>${yr.year}</b></td>
        <td><b style="color:${s.color};">●</b> ${s.name}</td>
        <td class="al-r ${stratClass}"><b>${yr.strategy_return_pct >= 0 ? '+' : ''}${yr.strategy_return_pct.toFixed(2)}%</b></td>
        <td class="al-r ${btcClass}">${yr.btc_return_pct >= 0 ? '+' : ''}${yr.btc_return_pct.toFixed(2)}%</td>
        <td class="al-r ${goldClass}">${yr.gold_return_pct >= 0 ? '+' : ''}${yr.gold_return_pct.toFixed(2)}%</td>
        <td class="al-r ${spClass}">${yr.sp500_return_pct >= 0 ? '+' : ''}${yr.sp500_return_pct.toFixed(2)}%</td>
        <td class="al-r ${alphaBtcClass}"><b>${yr.diff_vs_btc >= 0 ? '+' : ''}${yr.diff_vs_btc.toFixed(2)} pp</b></td>
        <td class="al-r ${alphaSpClass}"><b>${yr.diff_vs_sp500 >= 0 ? '+' : ''}${yr.diff_vs_sp500.toFixed(2)} pp</b></td>
      `;
      tbody.appendChild(tr);
    });
  });
}

function renderDetailKPIs() {
  const strat = payload.strategies.find(s => s.id === selectedStratId) || payload.strategies[0];
  if (!strat) return;

  const kpodsContainer = document.getElementById('kpods');
  const ch = strat.section_metrics;
  if (kpodsContainer && ch) {
    const blocks = [
      { key: 'is', title: 'IS (In-Sample)', data: ch.is_section, score: ch.score_is, pills: `<span class="pill gris">Score ${ch.score_is.toFixed(2)}</span>` },
      { key: 'oos', title: 'OOS (Out-Of-Sample)', data: ch.oos_section, score: ch.score_oos, pills: `<span class="pill gris">Score ${ch.score_oos.toFixed(2)}</span><span class="pill azul">Ret ${ch.retention_pct.toFixed(0)}%</span><span class="pill gris" title="Retención por velas">Ret/v ${ch.retention_velas_pct.toFixed(0)}%</span>` },
      { key: 'completo', title: 'COMPLETO (Período Total)', data: ch.completo_section, score: ch.score_completo, pills: `<span class="pill gold">Score COMP ${ch.score_completo.toFixed(2)}</span>` }
    ];

    const meses = n => (n >= 365 ? `≈ ${(n / 365).toFixed(1)} años` : `≈ ${(n / 30.4).toFixed(0)} meses`);

    kpodsContainer.innerHTML = blocks.map(b => {
      const q = b.data;
      const retSign = q.return_pct >= 0 ? '+' : '';
      const retClass = q.return_pct >= 0 ? 'pos' : 'neg';
      const pnlSign = q.pnl_usd >= 0 ? '+$' : '-$';
      const pnlAbs = Math.abs(q.pnl_usd).toLocaleString('en-US', { minimumFractionDigits: 0, maximumFractionDigits: 0 });
      const diffSign = q.diff_vs_bh_pct >= 0 ? '+' : '';
      const diffClass = q.diff_vs_bh_pct >= 0 ? 'pos' : 'neg';
      const bhRetSign = q.bh_return_pct >= 0 ? '+' : '';

      const ddVal = currentDdm === 'pico' ? q.max_dd_pico_pct : q.max_dd_pct;
      const ddEtq = currentDdm === 'pico' ? 'vs pico' : 'vs capital';

      return `
        <div class="pod">
          <div class="kpod-h">
            <h3>${b.title} <span class="muted" style="font-size:11px; font-weight:500;">${q.start_date} → ${q.end_date}</span></h3>
            <div class="chs">${b.pills}</div>
          </div>
          <div class="kgrid">
            <div class="kpi">
              <div class="k">Retorno</div>
              <div class="v ${retClass}">${retSign}${q.return_pct.toFixed(2)}%</div>
              <div class="s">PnL ${pnlSign}${pnlAbs}</div>
            </div>
            <div class="kpi">
              <div class="k">vs B&H (BTC)</div>
              <div class="v ${diffClass}">${diffSign}${q.diff_vs_bh_pct.toFixed(1)} pp</div>
              <div class="s">B&H ${bhRetSign}${q.bh_return_pct.toFixed(1)}%</div>
            </div>
            <div class="kpi">
              <div class="k">Trades</div>
              <div class="v">${q.trades}</div>
              <div class="s">${q.total_bars} velas</div>
            </div>
            <div class="kpi">
              <div class="k">Max DD</div>
              <div class="v neg">-${ddVal.toFixed(2)}%</div>
              <div class="s">${ddEtq} · -$${q.max_dd_usd.toFixed(0)}</div>
            </div>
            <div class="kpi">
              <div class="k">Max Estanc.</div>
              <div class="v gold">${q.max_stagnation_bars} velas</div>
              <div class="s">${meses(q.max_stagnation_bars)}</div>
            </div>
            <div class="kpi">
              <div class="k">B&H Max DD</div>
              <div class="v neg">-${q.bh_max_dd_pico_pct.toFixed(2)}%</div>
              <div class="s">vs pico histórico</div>
            </div>
          </div>
        </div>
      `;
    }).join('');
  }

  // Desglose del Score SWAP Fitness
  const fitnessCards = document.getElementById('fitness-cards');
  if (fitnessCards) {
    fitnessCards.innerHTML = `
      <div class="fitness-card">
        <h4>1. Ratio Ganancia / Max DD <span class="fitness-weight">50% Peso</span></h4>
        <p>Valora la eficiencia de capital frente al peor retroceso nominal registrado.</p>
        <div style="font-size:18px; font-weight:800; color:var(--fg-hi); font-family:var(--mono); margin-top:6px;">
          ${strat.metrics.capital_dd_ratio.toFixed(2)}x
        </div>
      </div>
      <div class="fitness-card">
        <h4>2. Linealidad R² Compuesta <span class="fitness-weight">25% Peso</span></h4>
        <p>Evalúa la regularidad de la curva de equidad (0.40 Global + 0.35 Promedio + 0.25 Mínimo).</p>
        <div style="font-size:18px; font-weight:800; color:var(--sp500); font-family:var(--mono); margin-top:6px;">
          R² = ${strat.metrics.composite_r2.toFixed(3)}
        </div>
      </div>
      <div class="fitness-card">
        <h4>3. Simetría Slope IS/OOS <span class="fitness-weight">25% Peso</span></h4>
        <p>Castiga drásticamente el sobreajuste si la pendiente decae fuera de muestra.</p>
        <div style="font-size:18px; font-weight:800; color:var(--gold); font-family:var(--mono); margin-top:6px;">
          Ratio = ${strat.metrics.slope_ratio.toFixed(3)}
        </div>
      </div>
    `;
  }
}

function renderTradesTable() {
  const strat = payload.strategies.find(s => s.id === selectedStratId) || payload.strategies[0];
  if (!strat) return;

  const countBadge = document.getElementById('trades-count-label');
  if (countBadge) {
    countBadge.innerHTML = `<span class="badge-item"><span class="badge-lbl">Total Swaps Ejecutados</span><span class="badge-val">${strat.trades.length} rotaciones</span></span>`;
  }

  const tbody = document.getElementById('swaps-body');
  if (!tbody) return;
  tbody.innerHTML = '';

  strat.trades.forEach(t => {
    const tr = document.createElement('tr');
    const pnlClass = t.pnl_pct >= 0 ? 'val-pos' : 'val-neg';
    const pnlSign = t.pnl_pct >= 0 ? '+' : '';

    const fromDot = getDotHtml(t.from_asset);
    const toDot = getDotHtml(t.to_asset);

    tr.innerHTML = `
      <td class="al-c">#${t.swap_index}</td>
      <td>${t.date}</td>
      <td class="al-c">${fromDot}</td>
      <td class="al-c" style="color:var(--dim);">→</td>
      <td class="al-c">${toDot}</td>
      <td class="al-r">$${t.exit_price < 10 ? t.exit_price.toFixed(4) : t.exit_price.toFixed(2)}</td>
      <td class="al-r">$${t.entry_price < 10 ? t.entry_price.toFixed(4) : t.entry_price.toFixed(2)}</td>
      <td class="al-r ${pnlClass}"><b>${pnlSign}${t.pnl_pct.toFixed(2)}%</b></td>
      <td class="al-r">$${t.fee_paid.toFixed(2)}</td>
      <td class="al-r"><b>$${t.capital_after.toFixed(2)}</b></td>
      <td class="al-c">${t.holding_bars} días</td>
      <td class="muted" style="max-width:260px; overflow:hidden; text-overflow:ellipsis;">${t.reason}</td>
    `;
    tbody.appendChild(tr);
  });
}

function getDotHtml(symbol) {
  switch (symbol) {
    case 'BTC': return '<span class="badge-dot dot-btc">BTC</span>';
    case 'GOLD': return '<span class="badge-dot dot-gold">ORO</span>';
    case 'SP500': return '<span class="badge-dot dot-sp500">SP500</span>';
    case 'SH_BTC': return '<span class="badge-dot dot-sh-btc">SH_BTC</span>';
    case 'SH_GOLD': return '<span class="badge-dot dot-sh-gold">SH_GOLD</span>';
    case 'SH_SP500': return '<span class="badge-dot dot-sh-sp500">SH_SPY</span>';
    default: return '<span class="badge-dot dot-cash">CASH</span>';
  }
}

function initHeadToHead() {
  const selA = document.getElementById('select-h2h-a');
  const selB = document.getElementById('select-h2h-b');
  const selC = document.getElementById('select-h2h-c');
  const btnSwap = document.getElementById('btn-h2h-swap');
  if (!selA || !selB || !payload.strategies || payload.strategies.length === 0) return;

  selA.innerHTML = '';
  selB.innerHTML = '';
  if (selC) {
    selC.innerHTML = '<option value="none">(Ninguno — Modo 2 Estrategias)</option>';
  }

  payload.strategies.forEach((s, idx) => {
    const medal = idx === 0 ? '🥇 ' : idx === 1 ? '🥈 ' : idx === 2 ? '🥉 ' : '';
    const optA = document.createElement('option');
    optA.value = s.id;
    optA.textContent = medal + s.name;
    selA.appendChild(optA);

    const optB = document.createElement('option');
    optB.value = s.id;
    optB.textContent = medal + s.name;
    selB.appendChild(optB);

    if (selC) {
      const optC = document.createElement('option');
      optC.value = s.id;
      optC.textContent = medal + s.name;
      selC.appendChild(optC);
    }
  });

  h2hSelectedIdA = payload.strategies[0].id;
  h2hSelectedIdB = payload.strategies.length > 1 ? payload.strategies[1].id : payload.strategies[0].id;
  h2hSelectedIdC = payload.strategies.length > 2 ? payload.strategies[2].id : 'none';

  selA.value = h2hSelectedIdA;
  selB.value = h2hSelectedIdB;
  if (selC) selC.value = h2hSelectedIdC;

  selA.addEventListener('change', (e) => {
    h2hSelectedIdA = e.target.value;
    renderHeadToHead();
  });

  selB.addEventListener('change', (e) => {
    h2hSelectedIdB = e.target.value;
    renderHeadToHead();
  });

  if (selC) {
    selC.addEventListener('change', (e) => {
      h2hSelectedIdC = e.target.value;
      renderHeadToHead();
    });
  }

  if (btnSwap) {
    btnSwap.addEventListener('click', () => {
      if (h2hSelectedIdC && h2hSelectedIdC !== 'none') {
        // Rotar cíclicamente A -> B -> C -> A
        const temp = h2hSelectedIdA;
        h2hSelectedIdA = h2hSelectedIdB;
        h2hSelectedIdB = h2hSelectedIdC;
        h2hSelectedIdC = temp;
        selA.value = h2hSelectedIdA;
        selB.value = h2hSelectedIdB;
        if (selC) selC.value = h2hSelectedIdC;
      } else {
        // Invertir A <-> B
        const temp = h2hSelectedIdA;
        h2hSelectedIdA = h2hSelectedIdB;
        h2hSelectedIdB = temp;
        selA.value = h2hSelectedIdA;
        selB.value = h2hSelectedIdB;
      }
      renderHeadToHead();
    });
  }

  renderHeadToHead();
}

function renderHeadToHead() {
  if (!payload.strategies || payload.strategies.length === 0) return;
  const stratA = payload.strategies.find(s => s.id === h2hSelectedIdA) || payload.strategies[0];
  const stratB = payload.strategies.find(s => s.id === (h2hSelectedIdB || payload.strategies[0].id)) || payload.strategies[0];
  const stratC = (h2hSelectedIdC && h2hSelectedIdC !== 'none') ? payload.strategies.find(s => s.id === h2hSelectedIdC) : null;

  // Actualizar encabezados de tabla
  const theadTr = document.getElementById('h2h-thead-tr');
  if (theadTr) {
    if (stratC) {
      theadTr.innerHTML = `
        <th class="al-l" style="width: 25%;">Métrica Cuantitativa</th>
        <th class="al-r" id="h2h-th-a" style="width: 20%;"><span class="dot" style="background:${stratA.color}"></span><span style="color:${stratA.color}">${stratA.name}</span></th>
        <th class="al-r" id="h2h-th-b" style="width: 20%;"><span class="dot" style="background:${stratB.color}"></span><span style="color:${stratB.color}">${stratB.name}</span></th>
        <th class="al-r" id="h2h-th-c" style="width: 20%;"><span class="dot" style="background:${stratC.color}"></span><span style="color:${stratC.color}">${stratC.name}</span></th>
        <th class="al-c" style="width: 15%;">Líder</th>
      `;
    } else {
      theadTr.innerHTML = `
        <th class="al-l" style="width: 28%;">Métrica Cuantitativa</th>
        <th class="al-r" id="h2h-th-a" style="width: 26%;"><span class="dot" style="background:${stratA.color}"></span><span style="color:${stratA.color}">${stratA.name}</span></th>
        <th class="al-c" id="h2h-th-mid" style="width: 20%;">Ventaja Relativa</th>
        <th class="al-r" id="h2h-th-b" style="width: 26%;"><span class="dot" style="background:${stratB.color}"></span><span style="color:${stratB.color}">${stratB.name}</span></th>
      `;
    }
  }

  // Tarjetas Resumen A, B y C
  const sumA = document.getElementById('h2h-summary-a');
  const sumB = document.getElementById('h2h-summary-b');
  const sumC = document.getElementById('h2h-summary-c');

  const renderCardContent = (s, tagClass, tagLabel) => {
    const retSign = s.metrics.total_return_pct >= 0 ? '+' : '';
    const retClass = s.metrics.total_return_pct >= 0 ? 'pos' : 'neg';
    return `
      <div class="h2h-card-head">
        <div class="h2h-card-title">
          <span class="h2h-tag ${tagClass}">${tagLabel}</span>
          <span class="dot" style="background:${s.color}"></span>
          <b>${s.name}</b>
        </div>
        <span class="pill gris">${s.architecture}</span>
        <span class="pill verde">${s.margin_mode || 'Capital Cruzado (Cross)'} (${s.leverage ? s.leverage + 'X' : '5X'})</span>
      </div>
      <div class="h2h-kpis-mini">
        <div class="kpi">
          <div class="k">Retorno Total</div>
          <div class="v ${retClass}">${retSign}${s.metrics.total_return_pct.toFixed(2)}%</div>
          <div class="s">CAGR ${s.metrics.cagr_pct.toFixed(2)}%</div>
        </div>
        <div class="kpi">
          <div class="k">Max Drawdown</div>
          <div class="v neg">-${s.metrics.max_drawdown_pct.toFixed(2)}%</div>
          <div class="s">${s.metrics.max_drawdown_bars} barras</div>
        </div>
        <div class="kpi">
          <div class="k">Sharpe / Sortino</div>
          <div class="v">${s.metrics.sharpe_ratio.toFixed(2)} / ${s.metrics.sortino_ratio.toFixed(2)}</div>
          <div class="s">Calmar ${s.metrics.calmar_ratio.toFixed(2)}</div>
        </div>
        <div class="kpi">
          <div class="k">SWAP Fitness</div>
          <div class="v gold">${s.metrics.swap_fitness_score.toFixed(2)} pts</div>
          <div class="s">Cap/DD ${s.metrics.capital_dd_ratio.toFixed(2)}x</div>
        </div>
        <div class="kpi">
          <div class="k">Linealidad R²</div>
          <div class="v">${s.metrics.composite_r2.toFixed(3)}</div>
          <div class="s">Slope ${s.metrics.slope_ratio.toFixed(3)}</div>
        </div>
        <div class="kpi">
          <div class="k">Swaps / WinRate</div>
          <div class="v">${s.metrics.total_swaps} (${s.metrics.win_rate_swaps_pct.toFixed(1)}%)</div>
          <div class="s">Comisiones $${s.metrics.total_fees_paid.toFixed(2)}</div>
        </div>
      </div>
    `;
  };

  if (sumA) sumA.innerHTML = renderCardContent(stratA, 'tag-a', 'Candidato A');
  if (sumB) sumB.innerHTML = renderCardContent(stratB, 'tag-b', 'Candidato B');
  if (sumC) {
    if (stratC) {
      sumC.style.display = 'flex';
      sumC.innerHTML = renderCardContent(stratC, 'tag-c', 'Candidato C');
    } else {
      sumC.style.display = 'none';
      sumC.innerHTML = '';
    }
  }

  // Matriz de Métricas Detallada Mano a Mano
  const tbody = document.getElementById('h2h-table-body');
  if (tbody) {
    const mA = stratA.metrics;
    const mB = stratB.metrics;
    const mC = stratC ? stratC.metrics : null;

    const formatAdvantage2 = (valA, valB, higherIsBetter, isPercentage) => {
      const diff = valA - valB;
      if (Math.abs(diff) < 1e-5) {
        return `<span class="adv-badge adv-tie">Empate</span>`;
      }
      const aWins = higherIsBetter ? diff > 0 : diff < 0;
      const winner = aWins ? 'A' : 'B';
      const badgeClass = aWins ? 'adv-a' : 'adv-b';
      const icon = aWins ? '🟢' : '🔵';

      let diffStr = '';
      if (isPercentage) {
        diffStr = `${Math.abs(diff).toFixed(2)}%`;
      } else {
        const base = Math.min(Math.abs(valA), Math.abs(valB));
        if (base > 1e-4) {
          const pctRel = (Math.abs(diff) / base) * 100.0;
          diffStr = `+${pctRel.toFixed(1)}%`;
        } else {
          diffStr = `${Math.abs(diff).toFixed(2)}`;
        }
      }

      return `<span class="adv-badge ${badgeClass}">${icon} ${winner} (${diffStr})</span>`;
    };

    const formatAdvantage3 = (valA, valB, valC, higherIsBetter) => {
      let best = 'A';
      let bestVal = valA;
      if (higherIsBetter) {
        if (valB > bestVal) { best = 'B'; bestVal = valB; }
        if (valC > bestVal) { best = 'C'; bestVal = valC; }
      } else {
        if (valB < bestVal) { best = 'B'; bestVal = valB; }
        if (valC < bestVal) { best = 'C'; bestVal = valC; }
      }
      const badgeClass = best === 'A' ? 'adv-a' : best === 'B' ? 'adv-b' : 'adv-c';
      const icon = best === 'A' ? '🟢' : best === 'B' ? '🔵' : '🟠';
      return `<span class="adv-badge ${badgeClass}">${icon} Líder: ${best}</span>`;
    };

    let rowsHtml = '';
    const addCat = (title) => {
      const colspan = stratC ? 5 : 4;
      rowsHtml += `<tr><td colspan="${colspan}" class="h2h-cat-header">${title}</td></tr>`;
    };

    const addRow = (label, displayA, displayB, displayC, valA, valB, valC, higherIsBetter, isPct) => {
      if (stratC) {
        const adv = formatAdvantage3(valA, valB, valC, higherIsBetter);
        rowsHtml += `
          <tr>
            <td class="al-l"><b>${label}</b></td>
            <td class="al-r">${displayA}</td>
            <td class="al-r">${displayB}</td>
            <td class="al-r">${displayC}</td>
            <td class="al-c">${adv}</td>
          </tr>
        `;
      } else {
        const adv = formatAdvantage2(valA, valB, higherIsBetter, isPct);
        rowsHtml += `
          <tr>
            <td class="al-l"><b>${label}</b></td>
            <td class="al-r">${displayA}</td>
            <td class="al-c">${adv}</td>
            <td class="al-r">${displayB}</td>
          </tr>
        `;
      }
    };

    addCat('1. Capital y Retorno');
    addRow('Capital Inicial', `$${stratA.initial_capital.toFixed(2)}`, `$${stratB.initial_capital.toFixed(2)}`, stratC ? `$${stratC.initial_capital.toFixed(2)}` : '', stratA.initial_capital, stratB.initial_capital, stratC ? stratC.initial_capital : 0, true, false);
    addRow('Capital Final', `$${stratA.final_capital.toFixed(2)}`, `$${stratB.final_capital.toFixed(2)}`, stratC ? `$${stratC.final_capital.toFixed(2)}` : '', stratA.final_capital, stratB.final_capital, stratC ? stratC.final_capital : 0, true, false);
    addRow('Ganancia Neta (PnL)', `$${stratA.net_profit.toFixed(2)}`, `$${stratB.net_profit.toFixed(2)}`, stratC ? `$${stratC.net_profit.toFixed(2)}` : '', stratA.net_profit, stratB.net_profit, stratC ? stratC.net_profit : 0, true, false);
    addRow('Retorno Total (%)', `${mA.total_return_pct >= 0 ? '+' : ''}${mA.total_return_pct.toFixed(2)}%`, `${mB.total_return_pct >= 0 ? '+' : ''}${mB.total_return_pct.toFixed(2)}%`, stratC ? `${mC.total_return_pct >= 0 ? '+' : ''}${mC.total_return_pct.toFixed(2)}%` : '', mA.total_return_pct, mB.total_return_pct, mC ? mC.total_return_pct : 0, true, true);
    addRow('CAGR Anualizado (%)', `${mA.cagr_pct.toFixed(2)}%`, `${mB.cagr_pct.toFixed(2)}%`, stratC ? `${mC.cagr_pct.toFixed(2)}%` : '', mA.cagr_pct, mB.cagr_pct, mC ? mC.cagr_pct : 0, true, true);

    addCat('2. Control de Riesgo');
    addRow('Máximo Drawdown (%)', `-${mA.max_drawdown_pct.toFixed(2)}%`, `-${mB.max_drawdown_pct.toFixed(2)}%`, stratC ? `-${mC.max_drawdown_pct.toFixed(2)}%` : '', mA.max_drawdown_pct, mB.max_drawdown_pct, mC ? mC.max_drawdown_pct : 0, false, true);
    addRow('Duración Máx DD (barras)', `${mA.max_drawdown_bars} barras`, `${mB.max_drawdown_bars} barras`, stratC ? `${mC.max_drawdown_bars} barras` : '', mA.max_drawdown_bars, mB.max_drawdown_bars, mC ? mC.max_drawdown_bars : 0, false, false);
    addRow('Sharpe Ratio', mA.sharpe_ratio.toFixed(2), mB.sharpe_ratio.toFixed(2), stratC ? mC.sharpe_ratio.toFixed(2) : '', mA.sharpe_ratio, mB.sharpe_ratio, mC ? mC.sharpe_ratio : 0, true, false);
    addRow('Sortino Ratio', mA.sortino_ratio.toFixed(2), mB.sortino_ratio.toFixed(2), stratC ? mC.sortino_ratio.toFixed(2) : '', mA.sortino_ratio, mB.sortino_ratio, mC ? mC.sortino_ratio : 0, true, false);
    addRow('Calmar Ratio', mA.calmar_ratio.toFixed(2), mB.calmar_ratio.toFixed(2), stratC ? mC.calmar_ratio.toFixed(2) : '', mA.calmar_ratio, mB.calmar_ratio, mC ? mC.calmar_ratio : 0, true, false);

    addCat('3. SWAP Fitness y Robustez');
    addRow('SWAP Fitness Score', `${mA.swap_fitness_score.toFixed(2)} pts`, `${mB.swap_fitness_score.toFixed(2)} pts`, stratC ? `${mC.swap_fitness_score.toFixed(2)} pts` : '', mA.swap_fitness_score, mB.swap_fitness_score, mC ? mC.swap_fitness_score : 0, true, false);
    addRow('Linealidad R² Compuesta', mA.composite_r2.toFixed(3), mB.composite_r2.toFixed(3), stratC ? mC.composite_r2.toFixed(3) : '', mA.composite_r2, mB.composite_r2, mC ? mC.composite_r2 : 0, true, false);
    addRow('Simetría Slope IS/OOS', mA.slope_ratio.toFixed(3), mB.slope_ratio.toFixed(3), stratC ? mC.slope_ratio.toFixed(3) : '', mA.slope_ratio, mB.slope_ratio, mC ? mC.slope_ratio : 0, true, false);
    addRow('Ratio Ganancia / Max DD USD', `${mA.capital_dd_ratio.toFixed(2)}x`, `${mB.capital_dd_ratio.toFixed(2)}x`, stratC ? `${mC.capital_dd_ratio.toFixed(2)}x` : '', mA.capital_dd_ratio, mB.capital_dd_ratio, mC ? mC.capital_dd_ratio : 0, true, false);

    addCat('4. Operativa de Cartera');
    addRow('Total Swaps / Rotaciones', `${mA.total_swaps}`, `${mB.total_swaps}`, stratC ? `${mC.total_swaps}` : '', mA.total_swaps, mB.total_swaps, mC ? mC.total_swaps : 0, false, false);
    addRow('Tasa de Acierto (WinRate)', `${mA.win_rate_swaps_pct.toFixed(1)}%`, `${mB.win_rate_swaps_pct.toFixed(1)}%`, stratC ? `${mC.win_rate_swaps_pct.toFixed(1)}%` : '', mA.win_rate_swaps_pct, mB.win_rate_swaps_pct, mC ? mC.win_rate_swaps_pct : 0, true, true);
    addRow('Comisiones + Slippage ($)', `$${mA.total_fees_paid.toFixed(2)}`, `$${mB.total_fees_paid.toFixed(2)}`, stratC ? `$${mC.total_fees_paid.toFixed(2)}` : '', mA.total_fees_paid, mB.total_fees_paid, mC ? mC.total_fees_paid : 0, false, false);

    addCat('5. Distribución de Exposición');
    addRow('Exposición Bitcoin (BTC)', `${mA.exposure_btc_pct.toFixed(1)}%`, `${mB.exposure_btc_pct.toFixed(1)}%`, stratC ? `${mC.exposure_btc_pct.toFixed(1)}%` : '', mA.exposure_btc_pct, mB.exposure_btc_pct, mC ? mC.exposure_btc_pct : 0, true, true);
    addRow('Exposición Oro (PAXG)', `${mA.exposure_gold_pct.toFixed(1)}%`, `${mB.exposure_gold_pct.toFixed(1)}%`, stratC ? `${mC.exposure_gold_pct.toFixed(1)}%` : '', mA.exposure_gold_pct, mB.exposure_gold_pct, mC ? mC.exposure_gold_pct : 0, true, true);
    addRow('Exposición S&P 500 (SPY)', `${mA.exposure_sp500_pct.toFixed(1)}%`, `${mB.exposure_sp500_pct.toFixed(1)}%`, stratC ? `${mC.exposure_sp500_pct.toFixed(1)}%` : '', mA.exposure_sp500_pct, mB.exposure_sp500_pct, mC ? mC.exposure_sp500_pct : 0, true, true);
    const shortA = mA.exposure_short_btc_pct + mA.exposure_short_gold_pct + mA.exposure_short_sp500_pct;
    const shortB = mB.exposure_short_btc_pct + mB.exposure_short_gold_pct + mB.exposure_short_sp500_pct;
    const shortC = mC ? mC.exposure_short_btc_pct + mC.exposure_short_gold_pct + mC.exposure_short_sp500_pct : 0;
    addRow('Exposición Posiciones Short', `${shortA.toFixed(1)}%`, `${shortB.toFixed(1)}%`, stratC ? `${shortC.toFixed(1)}%` : '', shortA, shortB, shortC, false, true);
    addRow('Preservación en Dólar (CASH)', `${mA.exposure_cash_pct.toFixed(1)}%`, `${mB.exposure_cash_pct.toFixed(1)}%`, stratC ? `${mC.exposure_cash_pct.toFixed(1)}%` : '', mA.exposure_cash_pct, mB.exposure_cash_pct, mC ? mC.exposure_cash_pct : 0, true, true);

    tbody.innerHTML = rowsHtml;
  }

  // Renderizar Gráfico de Barras Comparativo
  renderH2HBarChart(stratA, stratB, stratC);
}

function renderH2HBarChart(stratA, stratB, stratC) {
  const canvas = document.getElementById('chart-h2h-bar');
  if (!canvas) return;

  const ctx = canvas.getContext('2d');
  if (chartH2H) {
    chartH2H.destroy();
  }

  const mA = stratA.metrics;
  const mB = stratB.metrics;

  const labels = [
    'Retorno Total (%)',
    'Sharpe Ratio (x20)',
    'Sortino Ratio (x20)',
    'Resistencia DD (100-MaxDD%)',
    'SWAP Fitness Score',
    'Linealidad R² (x100)',
    'WinRate Rotaciones (%)'
  ];

  const dataA = [
    mA.total_return_pct,
    mA.sharpe_ratio * 20.0,
    mA.sortino_ratio * 20.0,
    Math.max(0, 100.0 - mA.max_drawdown_pct),
    mA.swap_fitness_score,
    mA.composite_r2 * 100.0,
    mA.win_rate_swaps_pct
  ];

  const dataB = [
    mB.total_return_pct,
    mB.sharpe_ratio * 20.0,
    mB.sortino_ratio * 20.0,
    Math.max(0, 100.0 - mB.max_drawdown_pct),
    mB.swap_fitness_score,
    mB.composite_r2 * 100.0,
    mB.win_rate_swaps_pct
  ];

  const datasets = [
    {
      label: 'A: ' + stratA.name,
      data: dataA,
      backgroundColor: 'rgba(0, 230, 118, 0.65)',
      borderColor: '#00E676',
      borderWidth: 1.5,
      borderRadius: 4
    },
    {
      label: 'B: ' + stratB.name,
      data: dataB,
      backgroundColor: 'rgba(0, 176, 255, 0.65)',
      borderColor: '#00B0FF',
      borderWidth: 1.5,
      borderRadius: 4
    }
  ];

  if (stratC) {
    const mC = stratC.metrics;
    const dataC = [
      mC.total_return_pct,
      mC.sharpe_ratio * 20.0,
      mC.sortino_ratio * 20.0,
      Math.max(0, 100.0 - mC.max_drawdown_pct),
      mC.swap_fitness_score,
      mC.composite_r2 * 100.0,
      mC.win_rate_swaps_pct
    ];
    datasets.push({
      label: 'C: ' + stratC.name,
      data: dataC,
      backgroundColor: 'rgba(255, 171, 0, 0.65)',
      borderColor: '#FFAB00',
      borderWidth: 1.5,
      borderRadius: 4
    });
  }

  chartH2H = new Chart(ctx, {
    type: 'bar',
    data: {
      labels: labels,
      datasets: datasets
    },
    options: {
      responsive: true,
      maintainAspectRatio: false,
      plugins: {
        legend: {
          labels: { color: '#EBE6E1', font: { family: 'Montserrat', size: 12, weight: 600 } }
        },
        tooltip: {
          backgroundColor: 'rgba(15, 18, 22, 0.95)',
          titleColor: '#FFFFFF',
          bodyColor: '#EBE6E1',
          borderColor: 'rgba(255,255,255,0.15)',
          borderWidth: 1,
          padding: 10
        }
      },
      scales: {
        x: {
          ticks: { color: '#9FAAB5', font: { family: 'Montserrat', size: 11 } },
          grid: { color: 'rgba(255,255,255,0.05)' }
        },
        y: {
          ticks: { color: '#9FAAB5', font: { family: 'JetBrains Mono', size: 11 } },
          grid: { color: 'rgba(255,255,255,0.05)' }
        }
      }
    }
  });
}
"#
}
