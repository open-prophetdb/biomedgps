# Cromwell API Wrapper for Rust

## What is Cromwell?

Cromwell is a scientific workflow engine designed for simplicity & scalability. Trivially transition between one off use cases to massive scale production environments. It supports multiple backends for storage and scheduling, such as Google Cloud Storage, AWS S3, and Google Compute Engine, Slurm, Local, and more.

## Notes

The cromwell server has removed support for Alibaba Cloud. The last version that supports it is 80. [See Release 81 Changelog](https://github.com/broadinstitute/cromwell/releases?q=81&expanded=true)

Luckily, Cromwell API is stable and compatible across versions. So as long as Cromwell 80 is deployed, the API calls should be compatible.

## Configuration

```conf
backend {
  default = "Local"
  providers {
    Local {
      actor-factory = "cromwell.backend.impl.local.LocalBackendLifecycleActorFactory"
      config {
        # Local backend's default configuration
        root = "cromwell-executions" # Cromwell将存储运行中的任务和工作流日志的位置
        filesystems {
          local {
            localization = [
              "hard-link", "soft-link", "copy"
            ]
          }
        }
      }
    }
  }
}

# 提供服务的路径
services {
  MetadataService {
    config {
      // Number of metadata entries to keep in memory. Helps optimize performance.
      metadata-summary-threshold = 2000
    }
  }

  # Call caching configuration, which allows Cromwell to skip re-running jobs that have the same inputs and workflow.
  CallCaching {
    enabled = false  # 默认关闭本地执行时的调用缓存
  }
}

# Database configuration to store workflow information. For local execution, using an H2 in-memory database is sufficient.
database {
  profile = "slick.jdbc.HsqldbProfile$"
  db {
    driver = "org.hsqldb.jdbcDriver"
    url = "jdbc:hsqldb:mem:workflowdb;shutdown=false"
    user = "sa"
    password = ""
  }
}

# Workflow logs and output configuration
system {
  workflow-log-dir = "workflow-logs" # 工作流日志的保存路径
  job-shell = "/bin/bash"  # 默认任务使用的 shell
}

# Server mode configuration. Since we're running Cromwell locally, there's no need to run it in server mode.
# Uncomment the lines below to configure Cromwell as a service:
#system {
#  cromwell-system {
#    dispatchers {
#      api-dispatcher {
#        type = Dispatcher
#        executor = "thread-pool-executor"
#      }
#    }
#  }
#}

# Enable the "soft linking" or "hard linking" of files instead of copying to improve performance.
filesystems {
  local {
    localization = [
      "hard-link", "soft-link", "copy"
    ]
  }
}
```

```ini
[Unit]
Description=Cromwell Workflow Engine
Documentation=https://cromwell.readthedocs.io/
After=network.target

[Service]
User=cromwell     # 运行 Cromwell 的用户
Group=cromwell    # 运行 Cromwell 的用户组
ExecStart=/usr/bin/java -Dconfig.file=/path/to/cromwell.conf -jar /path/to/cromwell.jar server
WorkingDirectory=/path/to/cromwell  # Cromwell 工作目录
Restart=on-failure
RestartSec=10
StandardOutput=append:/var/log/cromwell/cromwell.log
StandardError=append:/var/log/cromwell/cromwell.err.log
LimitNOFILE=65536  # 增加打开文件的限制，适用于可能会打开大量文件的工作流

[Install]
WantedBy=multi-user.target
```