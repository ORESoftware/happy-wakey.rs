-- Declarative Supabase schema for Happy Wakey.
-- Re-run this file whenever the desired schema changes; it is idempotent and
-- does not rely on a migrations table.

create schema if not exists private;

-- Table: user_config
-- Stores app configuration per user as JSONB
create table if not exists public.user_config (
  user_id uuid primary key default auth.uid(),
  config jsonb not null default '{}'::jsonb,
  updated_at timestamptz not null default now()
);

alter table public.user_config
  add column if not exists config jsonb not null default '{}'::jsonb,
  add column if not exists updated_at timestamptz not null default now();

-- Enable Row Level Security
alter table public.user_config enable row level security;
alter table public.user_config force row level security;

-- Policies: users can only read/write their own config
drop policy if exists "Users can read own config" on public.user_config;
create policy "Users can read own config"
  on public.user_config for select
  to authenticated
  using ((select auth.uid()) is not null and (select auth.uid()) = user_id);

drop policy if exists "Users can insert own config" on public.user_config;
create policy "Users can insert own config"
  on public.user_config for insert
  to authenticated
  with check ((select auth.uid()) is not null and (select auth.uid()) = user_id);

drop policy if exists "Users can update own config" on public.user_config;
create policy "Users can update own config"
  on public.user_config for update
  to authenticated
  using ((select auth.uid()) is not null and (select auth.uid()) = user_id)
  with check ((select auth.uid()) is not null and (select auth.uid()) = user_id);

revoke all on public.user_config from anon;
grant select, insert, update on public.user_config to authenticated;

-- Table: user_onboarding_state
-- Supabase is the authoritative sync target for onboarding progress.
create table if not exists public.user_onboarding_state (
  user_id uuid primary key default auth.uid(),
  completed boolean not null default false,
  current_step text not null default 'welcome'
    check (current_step in ('welcome', 'account', 'backup', 'essentials', 'ready', 'complete')),
  step_index smallint not null default 0
    check (step_index between 0 and 4),
  updated_at timestamptz not null default now()
);

alter table public.user_onboarding_state
  add column if not exists completed boolean not null default false,
  add column if not exists current_step text not null default 'welcome',
  add column if not exists step_index smallint not null default 0,
  add column if not exists updated_at timestamptz not null default now();

update public.user_onboarding_state
set
  current_step = case
    when completed then 'complete'
    when current_step in ('welcome', 'account', 'backup', 'essentials', 'ready') then current_step
    else 'welcome'
  end,
  step_index = least(greatest(step_index, 0), 4);

alter table public.user_onboarding_state
  drop constraint if exists user_onboarding_state_current_step_check,
  add constraint user_onboarding_state_current_step_check
    check (current_step in ('welcome', 'account', 'backup', 'essentials', 'ready', 'complete')),
  drop constraint if exists user_onboarding_state_step_index_check,
  add constraint user_onboarding_state_step_index_check
    check (step_index between 0 and 4);

alter table public.user_onboarding_state enable row level security;
alter table public.user_onboarding_state force row level security;

drop policy if exists "Users can read own onboarding state" on public.user_onboarding_state;
create policy "Users can read own onboarding state"
  on public.user_onboarding_state for select
  to authenticated
  using ((select auth.uid()) is not null and (select auth.uid()) = user_id);

drop policy if exists "Users can insert own onboarding state" on public.user_onboarding_state;
create policy "Users can insert own onboarding state"
  on public.user_onboarding_state for insert
  to authenticated
  with check ((select auth.uid()) is not null and (select auth.uid()) = user_id);

drop policy if exists "Users can update own onboarding state" on public.user_onboarding_state;
create policy "Users can update own onboarding state"
  on public.user_onboarding_state for update
  to authenticated
  using ((select auth.uid()) is not null and (select auth.uid()) = user_id)
  with check ((select auth.uid()) is not null and (select auth.uid()) = user_id);

revoke all on public.user_onboarding_state from anon;
grant select, insert, update on public.user_onboarding_state to authenticated;

-- Keep updated_at server-owned for REST upserts.
create or replace function private.set_updated_at()
returns trigger
language plpgsql
set search_path = ''
as $$
begin
  new.updated_at = now();
  return new;
end;
$$;

drop trigger if exists set_user_config_updated_at on public.user_config;
create trigger set_user_config_updated_at
  before insert or update on public.user_config
  for each row execute function private.set_updated_at();

drop trigger if exists set_user_onboarding_state_updated_at on public.user_onboarding_state;
create trigger set_user_onboarding_state_updated_at
  before insert or update on public.user_onboarding_state
  for each row execute function private.set_updated_at();

drop function if exists public.upsert_user_config(jsonb);
drop function if exists public.upsert_user_onboarding_state(boolean, text, smallint);
