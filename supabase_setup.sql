-- Run this in your Supabase SQL editor to set up the user_config table

-- Table: user_config
-- Stores app configuration per user as JSONB
create table if not exists public.user_config (
  user_id uuid primary key default auth.uid(),
  config jsonb not null default '{}'::jsonb,
  updated_at timestamptz not null default now()
);

-- Enable Row Level Security
alter table public.user_config enable row level security;

-- Policies: users can only read/write their own config
drop policy if exists "Users can read own config" on public.user_config;
create policy "Users can read own config"
  on public.user_config for select
  using (auth.uid() = user_id);

drop policy if exists "Users can insert own config" on public.user_config;
create policy "Users can insert own config"
  on public.user_config for insert
  with check (auth.uid() = user_id);

drop policy if exists "Users can update own config" on public.user_config;
create policy "Users can update own config"
  on public.user_config for update
  using (auth.uid() = user_id)
  with check (auth.uid() = user_id);

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

alter table public.user_onboarding_state enable row level security;

drop policy if exists "Users can read own onboarding state" on public.user_onboarding_state;
create policy "Users can read own onboarding state"
  on public.user_onboarding_state for select
  using (auth.uid() = user_id);

drop policy if exists "Users can insert own onboarding state" on public.user_onboarding_state;
create policy "Users can insert own onboarding state"
  on public.user_onboarding_state for insert
  with check (auth.uid() = user_id);

drop policy if exists "Users can update own onboarding state" on public.user_onboarding_state;
create policy "Users can update own onboarding state"
  on public.user_onboarding_state for update
  using (auth.uid() = user_id)
  with check (auth.uid() = user_id);

grant select, insert, update on public.user_onboarding_state to authenticated;

-- Function to upsert config (insert or update)
create or replace function public.upsert_user_config(p_config jsonb)
returns void
language plpgsql
security definer
as $$
begin
  insert into public.user_config (user_id, config, updated_at)
  values (auth.uid(), p_config, now())
  on conflict (user_id)
  do update set config = p_config, updated_at = now();
end;
$$;

create or replace function public.upsert_user_onboarding_state(
  p_completed boolean,
  p_current_step text,
  p_step_index smallint
)
returns void
language plpgsql
security definer
as $$
begin
  insert into public.user_onboarding_state (
    user_id,
    completed,
    current_step,
    step_index,
    updated_at
  )
  values (
    auth.uid(),
    p_completed,
    case when p_completed then 'complete' else p_current_step end,
    p_step_index,
    now()
  )
  on conflict (user_id)
  do update set
    completed = p_completed,
    current_step = case when p_completed then 'complete' else p_current_step end,
    step_index = p_step_index,
    updated_at = now();
end;
$$;
