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
create policy "Users can read own config"
  on public.user_config for select
  using (auth.uid() = user_id);

create policy "Users can insert own config"
  on public.user_config for insert
  with check (auth.uid() = user_id);

create policy "Users can update own config"
  on public.user_config for update
  using (auth.uid() = user_id);

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
