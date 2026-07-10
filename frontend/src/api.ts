import { SendFormValues } from './validation';

type SendApiSuccess = {
  status: string;
  message: string;
  delivery_status: string;
  message_id: string | null;
};

type ErrorBody = {
  code?: string;
  message?: string;
};

export class SendApiError extends Error {
  readonly code: string;
  readonly status: number;

  constructor(message: string, status: number, code: string) {
    super(message);
    this.name = 'SendApiError';
    this.status = status;
    this.code = code;
  }
}

export async function sendMessage(values: SendFormValues): Promise<SendApiSuccess> {
  let response: Response;

  try {
    response = await fetch('/api/send', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        recipient_email: values.recipientEmail,
        subject: values.subject,
        message: values.message,
      }),
    });
  } catch {
    throw new SendApiError(
      'Could not reach the send service. Try again later.',
      0,
      'network_error',
    );
  }

  const body = await readJson(response);

  if (!response.ok) {
    const error = errorBody(body);
    const code = error?.code ?? 'send_failed';

    throw new SendApiError(
      errorMessage(response.status, code, error?.message),
      response.status,
      code,
    );
  }

  return normalizeSuccess(body);
}

async function readJson(response: Response): Promise<unknown> {
  const text = await response.text();

  if (text.trim().length === 0) {
    return undefined;
  }

  try {
    return JSON.parse(text) as unknown;
  } catch {
    return undefined;
  }
}

function normalizeSuccess(body: unknown): SendApiSuccess {
  if (!isRecord(body)) {
    return {
      status: 'accepted',
      message: 'Message sent.',
      delivery_status: 'sent',
      message_id: null,
    };
  }

  return {
    status: stringValue(body.status, 'accepted'),
    message: stringValue(body.message, 'Message sent.'),
    delivery_status: stringValue(body.delivery_status, 'sent'),
    message_id: typeof body.message_id === 'string' ? body.message_id : null,
  };
}

function errorBody(body: unknown): ErrorBody | undefined {
  if (!isRecord(body)) {
    return undefined;
  }

  const error = body.error;
  if (!isRecord(error)) {
    return undefined;
  }

  return {
    code: typeof error.code === 'string' ? error.code : undefined,
    message: typeof error.message === 'string' ? error.message : undefined,
  };
}

function errorMessage(status: number, code: string, fallback?: string) {
  if (status === 429 || code === 'rate_limited' || code === 'email_rate_limited') {
    return 'Too many attempts. Try again later.';
  }

  if (code === 'validation_failed') {
    return 'Review the highlighted fields and try again.';
  }

  return fallback ?? 'Could not send the message. Try again later.';
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null;
}

function stringValue(value: unknown, fallback: string) {
  return typeof value === 'string' && value.trim().length > 0 ? value : fallback;
}
