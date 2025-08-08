# Raito Bridge Node Ansible Playbook

This Ansible playbook automates the deployment and management of the Raito Bridge Node on remote servers.

## Prerequisites

- Ansible 2.9 or higher
- SSH access to target machine(s)
- Python 3.6+ on both control machine and target machines
- Sudo privileges on target machines

## Features

- **Rust Installation**: Installs Rust toolchain if not present
- **Automated Installation**: Installs raito-bridge-node from the official repository
- **Service Management**: Creates and manages systemd service
- **Environment Configuration**: Securely manages Bitcoin RPC credentials
- **Update Management**: Provides clean update workflow (replaces binary without uninstall)
- **Service Control**: Easy start/stop operations
- **Data Fetch**: Fetch SQLite database from remote server to project root

## Directory Structure

```
scripts/deploy/
├── ansible.cfg          # Ansible configuration
├── inventory.yml        # Target hosts configuration
├── site.yml            # Main playbook
├── env.example         # Environment variables template
├── Makefile            # Convenient command shortcuts
├── README.md           # This file
└── roles/
    ├── setup/          # Initial installation and configuration
    │   ├── tasks/main.yml
    │   ├── templates/
    │   │   ├── env.j2
    │   │   └── raito-bridge-node.service.j2
    │   └── handlers/main.yml
    ├── start/          # Start and enable service
    │   └── tasks/main.yml
    ├── update/         # Update existing installation
    │   ├── tasks/main.yml
    │   └── templates/env.j2
    ├── stop/           # Stop and disable service
    │   └── tasks/main.yml
    ├── fetch_db/       # Fetch SQLite database
    │   └── tasks/main.yml
    └── expose/         # Set up HTTPS access via GCP Load Balancer
        └── tasks/main.yml
```

## Configuration

### 1. Environment Variables

Copy the example environment file and configure your settings:

```bash
cp env.example .env
```

Edit `.env` with your specific configuration:

```env
# Target machine configuration
TARGET_HOST=your.server.ip
TARGET_USER=ubuntu
SSH_KEY_PATH=~/.ssh/id_rsa

# Bitcoin RPC configuration
BITCOIN_RPC=http://your-bitcoin-node:8332
USERPWD=your_rpc_user:your_rpc_password

# Bridge node configuration (optional)
RPC_HOST=0.0.0.0:8080

# Google Cloud Platform configuration (for HTTPS exposure)
GCP_PROJECT_ID=your-project-id
GCP_ZONE=us-central1-a
GCP_DOMAIN=api.raito.wtf
GCP_INSTANCE_NAME=raito
```

### 2. Environment Variables (Automatic)

Environment variables are automatically loaded from the `.env` file when using the Makefile commands. If running ansible-playbook directly, you'll need to source the environment manually:

```bash
source .env
export $(cut -d= -f1 .env)
```

## Usage

### Initial Setup

Deploy and configure the Raito Bridge Node for the first time:

```bash
make setup
# OR: ansible-playbook site.yml --tags setup
```

This will:
- Create dedicated user account
- Install Rust if not present
- Install raito-bridge-node
- Configure and start systemd service

### Start Service

Start and enable the service:

```bash
make start
# OR: ansible-playbook site.yml --tags start
```

### Update Node

Update the Raito Bridge Node to the latest version:

```bash
make update
# OR: ansible-playbook site.yml --tags update
```

This will:
- Stop the current service
- Install the latest version (replaces existing binary)
- Restart the service

### Stop Service

Stop and disable the Raito Bridge Node service:

```bash
make stop
# OR: ansible-playbook site.yml --tags stop
```

### Expose via HTTPS

Set up HTTPS access to the bridge node API via Google Cloud Load Balancer:

```bash
make expose
# OR: ansible-playbook expose.yml
```

**Note**: This role runs locally using `gcloud` CLI and doesn't require SSH access to the target server. **No machine restart is required** - only GCP infrastructure is configured.

This will:
- Create managed SSL certificate for your domain
- Set up global load balancer with health checks

### Fetch Database

Fetch the SQLite database from the remote server to your local machine:

```bash
make fetch-db
# OR: ansible-playbook site.yml --tags fetch-db
```

This will:
- Ensure local `.mmr_data` directory exists in the project root
- Download the SQLite database (`mmr.db`) from remote server
- Provide status feedback for the download operation

**Note**: The database is downloaded to `.mmr_data/mmr.db` in the project root directory. This allows you to use the data locally for development or analysis purposes.
- Configure firewall rules
- Reserve static IP address (or use existing one)
- Provide DNS configuration instructions

## Service Management

Once deployed, you can manage the service directly on the target machine:

```bash
# Check service status
sudo systemctl status raito-bridge-node

# View logs
sudo journalctl -u raito-bridge-node -f

# Manual restart
sudo systemctl restart raito-bridge-node
```

## Security Features

- Dedicated system user with minimal privileges
- Secure file permissions for configuration files
- Systemd security hardening (NoNewPrivileges, PrivateTmp, etc.)
- Environment-based credential management

## File Locations

On the target machine:
- **Working Directory**: `/opt/raito-bridge-node/`
- **Configuration**: `/opt/raito-bridge-node/.env`
- **Data Storage**: `/opt/raito-bridge-node/.mmr_data/`
- **Service File**: `/etc/systemd/system/raito-bridge-node.service`
- **Binary Location**: `/home/raito/.cargo/bin/raito-bridge-node`

## Troubleshooting

### Check Service Status
```bash
make status
# OR: ansible all -m shell -a "systemctl status raito-bridge-node"
```

### View Logs
```bash
make logs
# OR: ansible all -m shell -a "journalctl -u raito-bridge-node --lines=50"
```

### Test Connectivity
```bash
make test-api
# OR: ansible all -m shell -a "curl -s http://localhost:8080/head"
```

## Storage Requirements

For the first 900K Bitcoin blocks:
- ~300MB for accumulator state database
- ~3.6GB for sparse roots files
- Additional space for logs and temporary files

## Commands Summary

- `make setup` - Initial deployment
- `make start` - Start service
- `make update` - Update to latest version  
- `make stop` - Stop service
- `make expose` - Set up HTTPS access via Google Cloud Load Balancer
- `make status` - Check status
- `make logs` - View logs
- `make test-api` - Test API endpoint

## HTTPS Setup Requirements

To use the `expose` command, you need:

1. **Google Cloud SDK**: Install the `gcloud` CLI tool
2. **Authentication**: Use gcloud CLI authentication
   ```bash
   gcloud auth login
   gcloud config set project your-project-id
   ```
4. **Domain**: Ensure your domain is properly configured to accept the SSL certificate

The expose command will:
- Create all necessary GCP resources
- Set up SSL certificate (managed by Google)
- Configure health checks for your bridge node
- Use existing static IP or create a new one
- Provide the static IP address for DNS configuration
- **No server downtime or restart required**

After running `make expose`, update your DNS records to point your domain to the provided static IP address.
