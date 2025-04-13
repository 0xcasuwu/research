#!/usr/bin/env node

/**
 * Visualization script for the bonding curve
 * 
 * This script generates a simple HTML file with a visualization of the bonding curve
 * using Chart.js.
 */

const fs = require('fs');
const path = require('path');

// Read the config file
const configPath = path.join(__dirname, '..', 'contracts', 'bonding-contract', 'config.json');
const config = JSON.parse(fs.readFileSync(configPath, 'utf8'));

// Get the environment from command line arguments
const args = process.argv.slice(2);
const env = args[0] || 'default';

if (!config.deployment[env]) {
  console.error(`Error: Environment '${env}' not found in config.json`);
  console.error(`Available environments: ${Object.keys(config.deployment).join(', ')}`);
  process.exit(1);
}

// Get the deployment configuration for the specified environment
const deploymentConfig = config.deployment[env];

// Calculate points for the bonding curve
function calculateCurvePoints(initialSupply, initialReserve, numPoints = 100, maxSupply = null) {
  const points = [];
  const maxSupplyValue = maxSupply || initialSupply * 2;
  const step = maxSupplyValue / numPoints;
  
  for (let i = 0; i <= numPoints; i++) {
    const supply = i * step;
    let price;
    
    if (supply === 0) {
      // Initial price
      price = initialReserve / (initialSupply * initialSupply);
    } else {
      price = initialReserve / (supply * supply);
    }
    
    points.push({ supply, price });
  }
  
  return points;
}

// Calculate points for buy and sell scenarios
function calculateScenarios(initialSupply, initialReserve) {
  const scenarios = [];
  
  // Buy scenarios
  const buyAmounts = [0.01, 0.05, 0.1, 0.2, 0.5].map(p => initialSupply * p);
  
  for (const amount of buyAmounts) {
    const newSupply = initialSupply + amount;
    const newReserve = initialReserve * (newSupply * newSupply) / (initialSupply * initialSupply);
    const dieselRequired = newReserve - initialReserve;
    
    scenarios.push({
      type: 'buy',
      tokenAmount: amount,
      dieselAmount: dieselRequired,
      pricePerToken: dieselRequired / amount,
      newSupply,
      newReserve,
      newPrice: newReserve / (newSupply * newSupply)
    });
  }
  
  // Sell scenarios
  const sellAmounts = [0.01, 0.05, 0.1, 0.2, 0.5].map(p => initialSupply * p);
  
  for (const amount of sellAmounts) {
    const newSupply = initialSupply - amount;
    const newReserve = initialReserve * (newSupply * newSupply) / (initialSupply * initialSupply);
    const dieselReceived = initialReserve - newReserve;
    
    scenarios.push({
      type: 'sell',
      tokenAmount: amount,
      dieselAmount: dieselReceived,
      pricePerToken: dieselReceived / amount,
      newSupply,
      newReserve,
      newPrice: newReserve / (newSupply * newSupply)
    });
  }
  
  return scenarios;
}

// Generate the HTML file
const curvePoints = calculateCurvePoints(deploymentConfig.initial_supply, deploymentConfig.initial_reserve);
const scenarios = calculateScenarios(deploymentConfig.initial_supply, deploymentConfig.initial_reserve);

const html = `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Bonding Curve Visualization (${env})</title>
  <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
  <style>
    body {
      font-family: Arial, sans-serif;
      margin: 0;
      padding: 20px;
      max-width: 1200px;
      margin: 0 auto;
    }
    .container {
      display: flex;
      flex-direction: column;
      gap: 20px;
    }
    .chart-container {
      height: 500px;
    }
    table {
      border-collapse: collapse;
      width: 100%;
    }
    th, td {
      border: 1px solid #ddd;
      padding: 8px;
      text-align: left;
    }
    th {
      background-color: #f2f2f2;
    }
    tr:nth-child(even) {
      background-color: #f9f9f9;
    }
    .buy {
      color: green;
    }
    .sell {
      color: red;
    }
  </style>
</head>
<body>
  <h1>Bonding Curve Visualization (${env})</h1>
  
  <div class="container">
    <div>
      <h2>Contract Configuration</h2>
      <p>
        <strong>Name:</strong> ${deploymentConfig.name}<br>
        <strong>Symbol:</strong> ${deploymentConfig.symbol}<br>
        <strong>Initial Supply:</strong> ${deploymentConfig.initial_supply}<br>
        <strong>Initial Reserve:</strong> ${deploymentConfig.initial_reserve}<br>
        <strong>Initial Price:</strong> ${deploymentConfig.initial_reserve / (deploymentConfig.initial_supply * deploymentConfig.initial_supply)}
      </p>
    </div>
    
    <div>
      <h2>Bonding Curve</h2>
      <div class="chart-container">
        <canvas id="curveChart"></canvas>
      </div>
    </div>
    
    <div>
      <h2>Buy Scenarios</h2>
      <table>
        <thead>
          <tr>
            <th>Token Amount</th>
            <th>Diesel Required</th>
            <th>Price Per Token</th>
            <th>New Supply</th>
            <th>New Reserve</th>
            <th>New Price</th>
          </tr>
        </thead>
        <tbody>
          ${scenarios
            .filter(s => s.type === 'buy')
            .map(s => `
              <tr>
                <td>${s.tokenAmount}</td>
                <td>${s.dieselAmount.toFixed(6)}</td>
                <td>${s.pricePerToken.toFixed(6)}</td>
                <td>${s.newSupply}</td>
                <td>${s.newReserve.toFixed(6)}</td>
                <td>${s.newPrice.toFixed(6)}</td>
              </tr>
            `).join('')}
        </tbody>
      </table>
    </div>
    
    <div>
      <h2>Sell Scenarios</h2>
      <table>
        <thead>
          <tr>
            <th>Token Amount</th>
            <th>Diesel Received</th>
            <th>Price Per Token</th>
            <th>New Supply</th>
            <th>New Reserve</th>
            <th>New Price</th>
          </tr>
        </thead>
        <tbody>
          ${scenarios
            .filter(s => s.type === 'sell')
            .map(s => `
              <tr>
                <td>${s.tokenAmount}</td>
                <td>${s.dieselAmount.toFixed(6)}</td>
                <td>${s.pricePerToken.toFixed(6)}</td>
                <td>${s.newSupply}</td>
                <td>${s.newReserve.toFixed(6)}</td>
                <td>${s.newPrice.toFixed(6)}</td>
              </tr>
            `).join('')}
        </tbody>
      </table>
    </div>
    
    <div>
      <h2>Slippage Analysis</h2>
      <p>
        Slippage occurs when the price of a token changes between the time a transaction is submitted and when it is executed.
        In the context of a bonding curve, slippage is inherent to the design and occurs because the price changes with each token bought or sold.
      </p>
      <p>
        For example, if you buy ${scenarios[2].tokenAmount} tokens and then immediately sell them:
      </p>
      <ul>
        <li>Buy price per token: ${scenarios[2].pricePerToken.toFixed(6)}</li>
        <li>Sell price per token: ${scenarios.filter(s => s.type === 'sell')[2].pricePerToken.toFixed(6)}</li>
        <li>Slippage: ${((1 - scenarios.filter(s => s.type === 'sell')[2].pricePerToken / scenarios[2].pricePerToken) * 100).toFixed(2)}%</li>
      </ul>
    </div>
  </div>
  
  <script>
    // Create the chart
    const ctx = document.getElementById('curveChart').getContext('2d');
    const chart = new Chart(ctx, {
      type: 'line',
      data: {
        datasets: [
          {
            label: 'Price',
            data: ${JSON.stringify(curvePoints.map(p => ({ x: p.supply, y: p.price })))},
            borderColor: 'blue',
            backgroundColor: 'rgba(0, 0, 255, 0.1)',
            fill: true,
            tension: 0.4
          },
          {
            label: 'Initial Point',
            data: [{ x: ${deploymentConfig.initial_supply}, y: ${deploymentConfig.initial_reserve / (deploymentConfig.initial_supply * deploymentConfig.initial_supply)} }],
            borderColor: 'red',
            backgroundColor: 'red',
            pointRadius: 5,
            pointHoverRadius: 7,
            showLine: false
          }
        ]
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        scales: {
          x: {
            type: 'linear',
            title: {
              display: true,
              text: 'Supply'
            }
          },
          y: {
            title: {
              display: true,
              text: 'Price'
            }
          }
        },
        plugins: {
          title: {
            display: true,
            text: 'Bonding Curve: price = reserve / (supply^2)'
          },
          tooltip: {
            callbacks: {
              label: function(context) {
                return \`Supply: \${context.parsed.x}, Price: \${context.parsed.y.toFixed(6)}\`;
              }
            }
          }
        }
      }
    });
  </script>
</body>
</html>`;

// Write the HTML file
const htmlPath = path.join(__dirname, '..', 'docs', `visualization-${env}.html`);
fs.writeFileSync(htmlPath, html);

console.log(`Visualization generated for environment '${env}': ${htmlPath}`);
