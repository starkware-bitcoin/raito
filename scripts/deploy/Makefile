.PHONY: help install setup start update stop expose status logs env-check

# Include .env file if it exists
-include .env
export

# Default target
help:
	@echo "Raito Bridge Node Ansible Playbook"
	@echo ""
	@echo "Available commands:"
	@echo "  install   - Install Ansible and required packages"
	@echo "  setup     - Initial deployment and configuration"
	@echo "  start     - Start and enable service"
	@echo "  update    - Update to latest version"
	@echo "  stop      - Stop and disable service"
	@echo "  expose    - Set up HTTPS access via Google Cloud Load Balancer"
	@echo "  status    - Check service status"
	@echo "  logs      - View service logs"
	@echo "  env-check - Verify environment variables"
	@echo ""
	@echo "Before running any command, make sure to:"
	@echo "  1. Copy env.example to .env and configure"
	@echo "  2. Environment variables will be automatically loaded from .env"

# Check if required environment variables are set
env-check:
	@echo "Checking environment variables..."
	@test -n "$$TARGET_HOST" || (echo "ERROR: TARGET_HOST not set" && exit 1)
	@test -n "$$BITCOIN_RPC" || (echo "ERROR: BITCOIN_RPC not set" && exit 1)
	@test -n "$$USERPWD" || (echo "ERROR: USERPWD not set" && exit 1)
	@echo "Environment variables OK"

# Install Ansible and required packages
install:
	@echo "Installing Ansible and required packages..."
	@which python3 > /dev/null || (echo "ERROR: python3 not found. Please install Python 3.6+" && exit 1)
	@which pip3 > /dev/null || (echo "ERROR: pip3 not found. Please install pip3" && exit 1)
	pip3 install --user ansible
	@echo "Ansible installation completed"
	@echo "Verifying installation..."
	ansible --version

# Initial setup
setup: env-check
	ansible-playbook site.yml --tags setup

# Start service
start: env-check
	ansible-playbook site.yml --tags start

# Update existing installation
update: env-check
	ansible-playbook site.yml --tags update

# Stop service
stop: env-check
	ansible-playbook site.yml --tags stop

# Set up HTTPS access
expose: env-check
	ansible-playbook expose.yml

# Check service status
status:
	ansible all -m shell -a "systemctl status raito-bridge-node --no-pager"

# View service logs
logs:
	ansible all -m shell -a "journalctl -u raito-bridge-node --lines=50 --no-pager"

# Test API endpoint
test-api:
	@echo "Testing API endpoint..."
	ansible all -m shell -a "curl -s http://localhost:8080/head || echo 'API not responding'"
