#!/usr/bin/env node

/**
 * Configuration script for the bonding contract
 * 
 * This script reads the config.json file and generates the necessary files
 * for deploying the bonding contract with the specified configuration.
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

// Convert string to hex representation for name and symbol
function stringToHex(str) {
  let hex = '0x';
  for (let i = 0; i < str.length; i++) {
    hex += str.charCodeAt(i).toString(16);
  }
  return hex;
}

// Generate the deployment script
const deploymentScript = `#!/bin/bash

# Deployment script for the bonding contract (${env} environment)
# Generated from config.json

set -e

# Check if the WASM file exists
if [ ! -f "dist/bonding_contract.wasm" ]; then
  echo "WASM file not found. Building the contract first..."
  ./scripts/build.sh
fi

echo "Deploying bonding contract (${env} environment)..."

# In a real deployment, you would use the Alkanes CLI or SDK to deploy the contract
# This is a placeholder for the actual deployment command
echo "This is a placeholder for the actual deployment command."
echo "In a real deployment, you would use the Alkanes CLI or SDK to deploy the contract."
echo ""
echo "Example deployment command:"
echo "alkanes deploy --wasm dist/bonding_contract.wasm --init-args \\"${stringToHex(deploymentConfig.name)},${stringToHex(deploymentConfig.symbol)},${deploymentConfig.initial_supply},${deploymentConfig.initial_reserve}\\""
echo ""
echo "The contract would be deployed with the following parameters:"
echo "  Name: ${deploymentConfig.name} (${stringToHex(deploymentConfig.name)})"
echo "  Symbol: ${deploymentConfig.symbol} (${stringToHex(deploymentConfig.symbol)})"
echo "  Initial Supply: ${deploymentConfig.initial_supply}"
echo "  Initial Reserve: ${deploymentConfig.initial_reserve}"
echo ""
echo "After deployment, the contract would be available at address [2, n],"
echo "where n is the next available sequence number."
`;

// Write the deployment script to a file
const deploymentScriptPath = path.join(__dirname, '..', 'scripts', `deploy-${env}.sh`);
fs.writeFileSync(deploymentScriptPath, deploymentScript);
fs.chmodSync(deploymentScriptPath, '755');

console.log(`Deployment script generated for environment '${env}': ${deploymentScriptPath}`);

// Generate a summary of the configuration
const summary = `# Bonding Contract Configuration (${env})

## Contract

- Name: ${config.contract.name}
- Version: ${config.contract.version}
- Description: ${config.contract.description}

## Deployment

- Name: ${deploymentConfig.name} (${stringToHex(deploymentConfig.name)})
- Symbol: ${deploymentConfig.symbol} (${stringToHex(deploymentConfig.symbol)})
- Initial Supply: ${deploymentConfig.initial_supply}
- Initial Reserve: ${deploymentConfig.initial_reserve}

## Bonding Curve

- Type: ${config.bonding_curve.type}
- Formula: ${config.bonding_curve.formula}

## Opcodes

${Object.entries(config.opcodes).map(([name, code]) => `- ${name}: ${code}`).join('\n')}
`;

// Write the summary to a file
const summaryPath = path.join(__dirname, '..', 'docs', `config-${env}.md`);
fs.writeFileSync(summaryPath, summary);

console.log(`Configuration summary generated for environment '${env}': ${summaryPath}`);

console.log('\nConfiguration complete!');
