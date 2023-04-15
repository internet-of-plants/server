CREATE TYPE Capability AS ENUM (
  'AllPastDay', 'PastWeekGroupedByHour', 'RestGroupedByDay', 'AllEvents', 'MultipleUsers', 'MultipleDevices'
);

CREATE TABLE IF NOT EXISTS capability_belongs_to_organization (
  capability      Capability  NOT NULL,
  organization_id BIGINT      NOT NULL,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE (capability, organization_id),
  FOREIGN KEY (organization_id) REFERENCES organizations (id)
);
