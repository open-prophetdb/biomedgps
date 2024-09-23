-- This file is used to create the tables for the workspace, workflow, task, and notification.

CREATE TABLE
    IF NOT EXISTS biomedgps_workspace (
        id VARCHAR(36) PRIMARY KEY,
        workspace_name VARCHAR(64),
        description TEXT,
        created_time TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_time TIMESTAMPTZ NOT NULL,
        archived_time TIMESTAMPTZ,
        payload JSONB NOT NULL, -- Any additional information for the workspace.
        owner VARCHAR(32) NOT NULL,
        groups VARCHAR(32)[] NOT NULL,

        CONSTRAINT biomedgps_workspace_uniq_key UNIQUE (workspace_name)
    );

-- The workflow table is used to store the information of workflows which are installed in the system.
CREATE TABLE
    IF NOT EXISTS biomedgps_workflow (
        id VARCHAR(32) PRIMARY KEY,
        workflow_name VARCHAR(255),
        icon TEXT,
        cover TEXT,
        description TEXT,
        repo_url TEXT,
        author VARCHAR(255),
        rate VARCHAR(16),
        valid BOOLEAN,
        version VARCHAR(255),
    );

CREATE TABLE
    IF NOT EXISTS biomedgps_task (
        id VARCHAR(36) PRIMARY KEY,
        workspace_id VARCHAR(36) NOT NULL, -- One workspace has many tasks.
        workflow_id VARCHAR(36) NOT NULL, -- One workflow has many tasks.
        task_id VARCHAR(36), -- One task has one task_id. We need to generate an uuid for tracking the task from the cromwell server.
        task_name VARCHAR(32) NOT NULL,
        description TEXT,
        submitted_time TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
        started_time TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
        finished_time TIMESTAMPTZ,
        task_params JSONB NOT NULL,
        labels JSONB NOT NULL,
        status VARCHAR(32) NOT NULL, -- The status of the task, such as running, finished, failed, etc.
        owner VARCHAR(32) NOT NULL,
        groups VARCHAR(32)[] NOT NULL,
        CONSTRAINT biomedgps_workflow_uniq_key UNIQUE (workspace_id, task_id)
    );

CREATE TABLE
    IF NOT EXISTS biomedgps_notification (
        id SERIAL NOT NULL,
        title VARCHAR(255) NOT NULL,
        description TEXT,
        notification_type VARCHAR(32) NOT NULL,
        created_time BIGINT NOT NULL,
        status VARCHAR(32) NOT NULL,
        owner VARCHAR(32) NOT NULL
    );