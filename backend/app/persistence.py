import os
from uuid import uuid4

import psycopg
from psycopg.types.json import Jsonb
from psycopg.rows import dict_row
import redis
from fastapi import APIRouter, HTTPException

DSN = os.getenv('APEIRON_DATABASE_URL', 'postgresql://apeiron:apeiron_dev@127.0.0.1:5432/apeiron')
cache = redis.Redis.from_url(os.getenv('APEIRON_REDIS_URL', 'redis://127.0.0.1:6379/0'), socket_timeout=1, socket_connect_timeout=1, decode_responses=True)
router = APIRouter(prefix='/api/archive')


def connect():
    return psycopg.connect(DSN, connect_timeout=3, row_factory=dict_row)


def initialize():
    with connect() as conn:
        conn.execute('''CREATE TABLE IF NOT EXISTS apeiron_records (
            id text PRIMARY KEY, kind text NOT NULL, name text NOT NULL,
            tick bigint, payload jsonb NOT NULL,
            created_at timestamptz NOT NULL DEFAULT now())''')
        conn.execute('CREATE INDEX IF NOT EXISTS records_kind_created ON apeiron_records (kind, created_at DESC)')


def save_record(kind, name, payload, tick=None):
    key = uuid4().hex
    with connect() as conn:
        if kind == 'checkpoint':
            conn.execute('SELECT pg_advisory_xact_lock(18001)')
            if conn.execute("SELECT count(*) AS n FROM apeiron_records WHERE kind = 'checkpoint'").fetchone()['n'] >= 8:
                raise HTTPException(409, 'Delete a checkpoint first (limit 8)')
        conn.execute('INSERT INTO apeiron_records (id, kind, name, tick, payload) VALUES (%s,%s,%s,%s,%s)', (key, kind, name, tick, Jsonb(payload)))
    return {'id': key, 'name': name, 'tick': tick}


def list_records(kind):
    with connect() as conn:
        return conn.execute('SELECT id, name, tick, created_at FROM apeiron_records WHERE kind = %s ORDER BY created_at DESC', (kind,)).fetchall()


def get_record(kind, key):
    # cache immutable research payloads only
    import json
    with connect() as conn:
        row = conn.execute('SELECT id, name, tick FROM apeiron_records WHERE kind = %s AND id = %s', (kind, key)).fetchone()
        if row is None:
            raise HTTPException(404, 'Record not found')
        if kind == 'research':
            try:
                cached = cache.get(f'apeiron:research:{key}')
                if cached is not None:
                    return row, json.loads(cached)
            except (redis.RedisError, ValueError):
                pass
        payload = conn.execute('SELECT payload FROM apeiron_records WHERE id = %s', (key,)).fetchone()
        if payload is None:
            raise HTTPException(404, 'Record not found')
        data = payload['payload']
    if kind == 'research':
        try:
            cache.setex(f'apeiron:research:{key}', 300, json.dumps(data))
        except redis.RedisError:
            pass
    return row, data


def delete_record(kind, key):
    with connect() as conn:
        if not conn.execute('DELETE FROM apeiron_records WHERE kind = %s AND id = %s', (kind, key)).rowcount:
            raise HTTPException(404, 'Record not found')
    try:
        cache.delete(f'apeiron:research:{key}')
    except redis.RedisError:
        pass
    return {'deleted': key}


@router.get('/health')
def health():
    result = {'postgres': False, 'redis': False}
    try:
        with connect() as conn:
            conn.execute('SELECT 1')
        result['postgres'] = True
    except psycopg.Error:
        pass
    try:
        result['redis'] = bool(cache.ping())
    except redis.RedisError:
        pass
    return result


@router.get('/research')
def research_list():
    return list_records('research')


@router.get('/research/{key}')
def research_get(key: str):
    return get_record('research', key)[1]


@router.delete('/research/{key}')
def research_delete(key: str):
    return delete_record('research', key)
