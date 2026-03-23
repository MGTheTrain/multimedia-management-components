#!/usr/bin/env bash
set -euo pipefail

SONAR_TOKEN="${SONAR_TOKEN:?SONAR_TOKEN env var is required}"
PROJECT_KEY="MGTheTrain_multimedia-management-components"

# Install sonar-scanner if not present
if ! command -v sonar-scanner &>/dev/null; then
  SONAR_VERSION=$(curl -s https://api.github.com/repos/SonarSource/sonar-scanner-cli/releases/latest |
    grep '"tag_name"' | cut -d'"' -f4)
  ARCH=$(uname -m)
  if [ "${ARCH}" = "aarch64" ] || [ "${ARCH}" = "arm64" ]; then
    SONAR_ARCH="linux-aarch64"
  else
    SONAR_ARCH="linux-x64"
  fi
  curl -sSLo /tmp/sonar-scanner.zip \
    "https://binaries.sonarsource.com/Distribution/sonar-scanner-cli/sonar-scanner-cli-${SONAR_VERSION}-${SONAR_ARCH}.zip"
  unzip -q /tmp/sonar-scanner.zip -d /tmp
  export PATH="/tmp/sonar-scanner-${SONAR_VERSION}-${SONAR_ARCH}/bin:$PATH"
fi

# Generate coverage
just coverage

# Run sonar-scanner
sonar-scanner

echo "Waiting for SonarCloud to process..."
sleep 10

echo "Checking quality thresholds..."
RESPONSE=$(curl -s \
  "https://sonarcloud.io/api/measures/component?component=${PROJECT_KEY}&metricKeys=coverage,reliability_rating,security_rating,sqale_rating")

python3 -c "
import sys, json

data = json.loads('${RESPONSE}')
measures = {m['metric']: m['value'] for m in data['component']['measures']}

coverage = float(measures['coverage'])
reliability = int(float(measures['reliability_rating']))
security = int(float(measures['security_rating']))
maintainability = int(float(measures['sqale_rating']))

failed = False
if coverage < 65.0:
    print(f'FAIL: Coverage {coverage}% is below 65%')
    failed = True
if reliability != 1:
    print(f'FAIL: Reliability rating is not A (got {reliability})')
    failed = True
if security != 1:
    print(f'FAIL: Security rating is not A (got {security})')
    failed = True
if maintainability != 1:
    print(f'FAIL: Maintainability rating is not A (got {maintainability})')
    failed = True

if failed:
    sys.exit(1)
print(f'All checks passed — Coverage: {coverage}%, Reliability: A, Security: A, Maintainability: A')
"

echo "Done. Check https://sonarcloud.io for results."
