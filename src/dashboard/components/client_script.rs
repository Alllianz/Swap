pub fn get_client_javascript() -> &'static str {
    r#"
let payload = null;
let chartCurvas = null;
let chartDrawdowns = null;
let chartAsignacion = null;
let currentEscala = 'log';
let currentRango = 'todo';
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
      selectedStratId = e.target.value;
      renderDetailKPIs();
      renderTradesTable();
      if (selectAlloc) selectAlloc.value = selectedStratId;
      renderAsignacionChart(selectedStratId);
    });
  }
}

function initLeaderboard() {
  const tbody = document.getElementById('leaderboard-body');
  if (!tbody) return;
  tbody.innerHTML = '';

  // 1. Renderizar Estrategias SWAP
  payload.strategies.forEach(s => {
    const tr = document.createElement('tr');
    tr.style.cursor = 'pointer';
    tr.onclick = () => {
      selectedStratId = s.id;
      const sd = document.getElementById('select-strat-detail');
      if (sd) sd.value = s.id;
      const sa = document.getElementById('select-strat-allocation');
      if (sa) sa.value = s.id;
      renderDetailKPIs();
      renderTradesTable();
      renderAsignacionChart(s.id);
      document.getElementById('detalle').scrollIntoView({ behavior: 'smooth' });
    };

    const retClass = s.metrics.total_return_pct >= 0 ? 'val-pos' : 'val-neg';
    const retSign = s.metrics.total_return_pct >= 0 ? '+' : '';

    tr.innerHTML = `
      <td><b style="color:${s.color};">●</b> <b>${s.name}</b></td>
      <td><span class="tag-badge">${s.architecture}</span></td>
      <td class="al-r ${retClass}"><b>${retSign}${s.metrics.total_return_pct.toFixed(2)}%</b></td>
      <td class="al-r"><b>${s.metrics.cagr_pct.toFixed(2)}%</b></td>
      <td class="al-r"><b>${s.metrics.sharpe_ratio.toFixed(2)}</b></td>
      <td class="al-r">${s.metrics.sortino_ratio.toFixed(2)}</td>
      <td class="al-r val-neg">-${s.metrics.max_drawdown_pct.toFixed(2)}%</td>
      <td class="al-r">${s.metrics.calmar_ratio.toFixed(2)}</td>
      <td class="al-r"><b style="color:var(--sp500);">${s.metrics.swap_fitness_score.toFixed(2)}</b></td>
      <td class="al-r">${s.metrics.total_swaps}</td>
      <td class="al-r">$${s.metrics.total_fees_paid.toFixed(2)}</td>
      <td class="al-c">
        <span class="badge-dot dot-btc">BTC ${s.metrics.exposure_btc_pct.toFixed(0)}%</span>
        <span class="badge-dot dot-gold">ORO ${s.metrics.exposure_gold_pct.toFixed(0)}%</span>
        <span class="badge-dot dot-sp500">SPY ${s.metrics.exposure_sp500_pct.toFixed(0)}%</span>
        <span class="badge-dot dot-cash">CASH ${s.metrics.exposure_cash_pct.toFixed(0)}%</span>
      </td>
    `;
    tbody.appendChild(tr);
  });

  // 2. Separador
  const sepTr = document.createElement('tr');
  sepTr.innerHTML = `<td colspan="12" style="background:rgba(255,255,255,0.02); font-weight:700; color:var(--dim); font-size:10.5px; text-transform:uppercase; letter-spacing:0.08em; padding:6px 12px;">── Benchmarks Pasivos de Referencia (Buy & Hold) ──</td>`;
  tbody.appendChild(sepTr);

  // 3. Renderizar Benchmarks
  payload.benchmarks.forEach(b => {
    const tr = document.createElement('tr');
    const retClass = b.total_return_pct >= 0 ? 'val-pos' : 'val-neg';
    const retSign = b.total_return_pct >= 0 ? '+' : '';

    tr.innerHTML = `
      <td><b style="color:${b.color};">●</b> ${b.name}</td>
      <td><span class="muted">Benchmark Pasivo</span></td>
      <td class="al-r ${retClass}"><b>${retSign}${b.total_return_pct.toFixed(2)}%</b></td>
      <td class="al-r"><b>${b.cagr_pct.toFixed(2)}%</b></td>
      <td class="al-r"><b>${b.sharpe_ratio.toFixed(2)}</b></td>
      <td class="al-r">-</td>
      <td class="al-r val-neg">-${b.max_drawdown_pct.toFixed(2)}%</td>
      <td class="al-r">-</td>
      <td class="al-r">-</td>
      <td class="al-r">0</td>
      <td class="al-r">$0.00</td>
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

  // 1. Configurar C1: Curvas Base 100
  const ctxCurvas = document.getElementById('chart-curvas').getContext('2d');
  const datasetsCurvas = [];

  // Añadir estrategias
  payload.strategies.forEach(s => {
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

  // Añadir benchmarks
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

  // 2. Configurar C2: Drawdowns Superpuestos
  const ctxDrawdowns = document.getElementById('chart-drawdowns').getContext('2d');
  const datasetsDrawdown = [];

  payload.strategies.forEach(s => {
    datasetsDrawdown.push({
      label: s.name,
      borderColor: s.color,
      backgroundColor: s.color,
      borderWidth: 2,
      pointRadius: 0,
      data: s.drawdown_curve.slice(startIdx, endIdx).map(d => -d),
      tension: 0.1,
    });
  });

  payload.benchmarks.forEach(b => {
    datasetsDrawdown.push({
      label: b.name,
      borderColor: b.color,
      backgroundColor: b.color,
      borderWidth: 1.2,
      borderDash: [3, 3],
      pointRadius: 0,
      data: b.drawdown_curve.slice(startIdx, endIdx).map(d => -d),
      tension: 0.1,
    });
  });

  chartDrawdowns = new Chart(ctxDrawdowns, {
    type: 'line',
    data: {
      labels: visibleDates,
      datasets: datasetsDrawdown,
    },
    options: {
      responsive: true,
      maintainAspectRatio: false,
      interaction: { mode: 'index', intersect: false },
      plugins: {
        legend: {
          display: true,
          position: 'top',
          labels: { color: '#9FAAB5', boxWidth: 12, font: { size: 10, weight: 'bold' } }
        },
        tooltip: {
          callbacks: {
            label: (ctx) => `${ctx.dataset.label}: ${ctx.parsed.y.toFixed(2)}%`
          }
        }
      },
      scales: {
        x: {
          grid: { color: 'rgba(255,255,255,0.05)' },
          ticks: { color: '#6C7885', maxTicksLimit: 12, font: { family: 'JetBrains Mono', size: 10 } }
        },
        y: {
          min: -85,
          max: 0,
          grid: { color: 'rgba(255,255,255,0.05)' },
          ticks: {
            color: '#FF2E4D',
            font: { family: 'JetBrains Mono', size: 10 },
            callback: (v) => `${v}%`
          }
        }
      }
    }
  });

  // 3. Render inicial de Gráfico C3 Asignación
  renderAsignacionChart(selectedStratId);
}

function updateChartsRange() {
  const [startIdx, endIdx] = getRangeIndices();
  const visibleDates = payload.dates.slice(startIdx, endIdx);

  // Actualizar Curvas
  if (chartCurvas) {
    chartCurvas.data.labels = visibleDates;
    chartCurvas.data.datasets.forEach(ds => {
      let raw = null;
      const strat = payload.strategies.find(s => s.id === ds.id);
      if (strat) {
        raw = strat.equity_curve.slice(startIdx, endIdx);
      } else {
        const b = payload.benchmarks.find(bm => bm.id === ds.id);
        if (b) raw = b.equity_curve.slice(startIdx, endIdx);
      }
      if (raw) {
        const baseVal = raw[0] || 1.0;
        ds.data = raw.map(v => (v / baseVal) * 100.0);
      }
    });
    chartCurvas.update();
  }

  // Actualizar Drawdowns
  if (chartDrawdowns) {
    chartDrawdowns.data.labels = visibleDates;
    chartDrawdowns.data.datasets.forEach(ds => {
      const strat = payload.strategies.find(s => s.name === ds.label);
      if (strat) {
        ds.data = strat.drawdown_curve.slice(startIdx, endIdx).map(d => -d);
      } else {
        const b = payload.benchmarks.find(bm => bm.name === ds.label);
        if (b) ds.data = b.drawdown_curve.slice(startIdx, endIdx).map(d => -d);
      }
    });
    chartDrawdowns.update();
  }

  // Actualizar Asignación
  renderAsignacionChart(selectedStratId);
}

function renderAsignacionChart(stratId) {
  const id = stratId || selectedStratId || (payload.strategies[0] ? payload.strategies[0].id : null);
  if (!id) return;

  const strat = payload.strategies.find(s => s.id === id);
  if (!strat) return;

  const [startIdx, endIdx] = getRangeIndices();
  const visibleDates = payload.dates.slice(startIdx, endIdx);
  const visibleAlloc = strat.allocations.slice(startIdx, endIdx);

  // Mapear activo a valor numérico escalonado: CASH=0, SP500=1, GOLD=2, BTC=3
  const assetVals = visibleAlloc.map(a => {
    switch (a.asset) {
      case 'BTC': return 3;
      case 'GOLD': return 2;
      case 'SP500': return 1;
      default: return 0; // CASH
    }
  });

  const ctx = document.getElementById('chart-asignacion').getContext('2d');
  if (chartAsignacion) {
    chartAsignacion.destroy();
  }

  chartAsignacion = new Chart(ctx, {
    type: 'line',
    data: {
      labels: visibleDates,
      datasets: [{
        label: `Asignación de Activo (${strat.name})`,
        data: assetVals,
        stepped: 'before',
        borderColor: strat.color,
        borderWidth: 2,
        fill: true,
        backgroundColor: 'rgba(0, 176, 255, 0.08)',
        pointRadius: 0,
      }]
    },
    options: {
      responsive: true,
      maintainAspectRatio: false,
      plugins: {
        legend: { display: false },
        tooltip: {
          callbacks: {
            label: (ctx) => {
              const val = ctx.parsed.y;
              const names = ['CASH (Liquidez / USDT)', 'S&P 500 (SPY)', 'ORO (Gold / PAXG)', 'Bitcoin (BTC)'];
              return `Activo en Cartera: ${names[val] || 'CASH'}`;
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
          min: -0.2,
          max: 3.2,
          ticks: {
            stepSize: 1,
            color: '#FFFFFF',
            font: { family: 'Montserrat', size: 11, weight: 'bold' },
            callback: (val) => {
              switch (val) {
                case 3: return '🟠 BTC';
                case 2: return '🟡 ORO';
                case 1: return '🔵 SP500';
                case 0: return '🟢 CASH';
                default: return '';
              }
            }
          },
          grid: { color: 'rgba(255,255,255,0.1)' }
        }
      }
    }
  });
}

function renderYearlyTable() {
  const tbody = document.getElementById('anual-body');
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

  const kpisContainer = document.getElementById('kpis-container');
  if (kpisContainer) {
    kpisContainer.innerHTML = `
      <div class="kpi-card">
        <span class="kpi-title">Capital Final</span>
        <span class="kpi-value">$${strat.final_capital.toLocaleString('en-US', {minimumFractionDigits: 2, maximumFractionDigits: 2})}</span>
        <span class="kpi-sub">Inicial: $${strat.initial_capital.toLocaleString()}</span>
      </div>
      <div class="kpi-card">
        <span class="kpi-title">Retorno Total / CAGR</span>
        <span class="kpi-value val-pos">+${strat.metrics.total_return_pct.toFixed(2)}%</span>
        <span class="kpi-sub">CAGR: +${strat.metrics.cagr_pct.toFixed(2)}% anual</span>
      </div>
      <div class="kpi-card">
        <span class="kpi-title">Sharpe / Sortino</span>
        <span class="kpi-value" style="color:var(--sp500);">${strat.metrics.sharpe_ratio.toFixed(2)}</span>
        <span class="kpi-sub">Sortino: ${strat.metrics.sortino_ratio.toFixed(2)}</span>
      </div>
      <div class="kpi-card">
        <span class="kpi-title">Máximo Drawdown</span>
        <span class="kpi-value val-neg">-${strat.metrics.max_drawdown_pct.toFixed(2)}%</span>
        <span class="kpi-sub">Duración: ${strat.metrics.max_drawdown_bars} velas</span>
      </div>
      <div class="kpi-card">
        <span class="kpi-title">Score SWAP Fitness</span>
        <span class="kpi-value" style="color:var(--gold);">${strat.metrics.swap_fitness_score.toFixed(2)}</span>
        <span class="kpi-sub">Calmar Ratio: ${strat.metrics.calmar_ratio.toFixed(2)}</span>
      </div>
      <div class="kpi-card">
        <span class="kpi-title">Rotaciones (Swaps)</span>
        <span class="kpi-value">${strat.metrics.total_swaps}</span>
        <span class="kpi-sub">Win Rate: ${strat.metrics.win_rate_swaps_pct.toFixed(1)}% | Fees: $${strat.metrics.total_fees_paid.toFixed(2)}</span>
      </div>
    `;
  }

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
    default: return '<span class="badge-dot dot-cash">CASH</span>';
  }
}
"#
}
