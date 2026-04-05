-- chesstui multiplayer schema

CREATE TABLE public.users (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email         TEXT UNIQUE NOT NULL,
    display_name  TEXT,
    elo           INTEGER NOT NULL DEFAULT 1200,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE public.sessions (
    token       TEXT PRIMARY KEY,
    user_id     UUID NOT NULL REFERENCES public.users(id),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at  TIMESTAMPTZ NOT NULL
);

CREATE TABLE public.otp_codes (
    email       TEXT NOT NULL,
    code        TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at  TIMESTAMPTZ NOT NULL,
    used        BOOLEAN NOT NULL DEFAULT FALSE
);
CREATE INDEX idx_otp_email ON public.otp_codes(email, used);

CREATE TABLE public.games (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    white_id      UUID NOT NULL REFERENCES public.users(id),
    black_id      UUID NOT NULL REFERENCES public.users(id),
    result        TEXT,
    result_detail TEXT,
    moves_json    JSONB NOT NULL DEFAULT '[]',
    started_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    finished_at   TIMESTAMPTZ
);
CREATE INDEX idx_games_white ON public.games(white_id);
CREATE INDEX idx_games_black ON public.games(black_id);
