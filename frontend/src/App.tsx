import { FormEvent, useMemo, useState } from 'react';

import { SendApiError, sendMessage } from './api';
import { SendFormValues, validateSendForm } from './validation';

const emptyForm: SendFormValues = {
  recipientEmail: '',
  subject: '',
  message: '',
};

const emptyTouched = {
  recipientEmail: false,
  subject: false,
  message: false,
};

type FieldName = keyof SendFormValues;

export function App() {
  const [values, setValues] = useState<SendFormValues>(emptyForm);
  const [touched, setTouched] = useState<Record<FieldName, boolean>>(emptyTouched);
  const [submitted, setSubmitted] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [submitError, setSubmitError] = useState<string | undefined>();
  const [successMessage, setSuccessMessage] = useState<string | undefined>();

  const errors = useMemo(() => validateSendForm(values), [values]);
  const hasErrors = Object.values(errors).some(Boolean);

  function updateField(field: FieldName, value: string) {
    setSubmitError(undefined);
    setSuccessMessage(undefined);
    setValues((current) => ({
      ...current,
      [field]: value,
    }));
  }

  function markTouched(field: FieldName) {
    setTouched((current) => ({
      ...current,
      [field]: true,
    }));
  }

  function fieldError(field: FieldName) {
    if (!submitted && !touched[field]) {
      return undefined;
    }

    return errors[field];
  }

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setSubmitted(true);
    setSubmitError(undefined);
    setSuccessMessage(undefined);
    setTouched({
      recipientEmail: true,
      subject: true,
      message: true,
    });

    if (hasErrors || isSubmitting) {
      return;
    }

    setIsSubmitting(true);
    try {
      const response = await sendMessage(values);
      setValues(emptyForm);
      setTouched(emptyTouched);
      setSubmitted(false);
      setSuccessMessage(response.message);
    } catch (error) {
      setSubmitError(
        error instanceof SendApiError
          ? error.message
          : 'Could not send the message. Try again later.',
      );
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <main className="min-h-screen bg-stone-100 text-stone-950">
      <section className="mx-auto grid min-h-screen w-full max-w-6xl items-center gap-8 px-5 py-8 md:grid-cols-[0.8fr_1.2fr] md:px-8">
        <div className="space-y-5">
          <p className="text-sm font-semibold uppercase tracking-wide text-teal-700">
            Message delivery
          </p>
          <h1 className="max-w-xl text-4xl font-semibold leading-tight tracking-normal text-stone-950 sm:text-5xl">
            Send a message.
          </h1>
          <p className="max-w-md text-base leading-7 text-stone-600">
            Enter the recipient, subject, and message body before sending.
          </p>
        </div>

        <form
          className="w-full rounded-lg border border-stone-300 bg-white p-5 shadow-sm sm:p-6"
          noValidate
          onSubmit={handleSubmit}
        >
          <div className="space-y-5">
            <StatusMessage error={submitError} success={successMessage} />

            <FormField
              error={fieldError('recipientEmail')}
              inputMode="email"
              label="Recipient email"
              name="recipientEmail"
              onBlur={() => markTouched('recipientEmail')}
              onChange={(value) => updateField('recipientEmail', value)}
              placeholder="name@example.com"
              type="email"
              value={values.recipientEmail}
            />

            <FormField
              error={fieldError('subject')}
              label="Subject"
              name="subject"
              onBlur={() => markTouched('subject')}
              onChange={(value) => updateField('subject', value)}
              placeholder="Subject"
              value={values.subject}
            />

            <MessageField
              error={fieldError('message')}
              onBlur={() => markTouched('message')}
              onChange={(value) => updateField('message', value)}
              value={values.message}
            />

            <div className="flex flex-col gap-3 border-t border-stone-200 pt-5 sm:flex-row sm:items-center sm:justify-between">
              <p className="min-h-5 text-sm text-stone-500" aria-live="polite">
                {submitted && hasErrors ? 'Review the highlighted fields.' : ''}
              </p>
              <button
                className="inline-flex h-11 items-center justify-center rounded-md bg-teal-700 px-5 text-sm font-semibold text-white shadow-sm transition hover:bg-teal-800 focus:outline-none focus:ring-2 focus:ring-teal-700 focus:ring-offset-2 disabled:cursor-not-allowed disabled:bg-stone-400"
                disabled={isSubmitting}
                type="submit"
              >
                {isSubmitting ? 'Sending...' : 'Send'}
              </button>
            </div>
          </div>
        </form>
      </section>
    </main>
  );
}

function StatusMessage({
  error,
  success,
}: {
  error?: string;
  success?: string;
}) {
  if (!error && !success) {
    return null;
  }

  const isError = Boolean(error);

  return (
    <div
      className={`rounded-md border px-4 py-3 text-sm ${
        isError
          ? 'border-rose-200 bg-rose-50 text-rose-800'
          : 'border-emerald-200 bg-emerald-50 text-emerald-800'
      }`}
      role={isError ? 'alert' : 'status'}
    >
      {error ?? success}
    </div>
  );
}

type FormFieldProps = {
  error?: string;
  inputMode?: 'email' | 'text';
  label: string;
  name: FieldName;
  onBlur: () => void;
  onChange: (value: string) => void;
  placeholder: string;
  type?: 'email' | 'text';
  value: string;
};

function FormField({
  error,
  inputMode = 'text',
  label,
  name,
  onBlur,
  onChange,
  placeholder,
  type = 'text',
  value,
}: FormFieldProps) {
  const errorId = `${name}-error`;

  return (
    <label className="block" htmlFor={name}>
      <span className="mb-2 block text-sm font-medium text-stone-800">{label}</span>
      <input
        aria-describedby={error ? errorId : undefined}
        aria-invalid={Boolean(error)}
        className={inputClassName(Boolean(error))}
        id={name}
        inputMode={inputMode}
        name={name}
        onBlur={onBlur}
        onChange={(event) => onChange(event.target.value)}
        placeholder={placeholder}
        type={type}
        value={value}
      />
      <FieldError id={errorId} message={error} />
    </label>
  );
}

type MessageFieldProps = {
  error?: string;
  onBlur: () => void;
  onChange: (value: string) => void;
  value: string;
};

function MessageField({ error, onBlur, onChange, value }: MessageFieldProps) {
  return (
    <label className="block" htmlFor="message">
      <span className="mb-2 block text-sm font-medium text-stone-800">Message</span>
      <textarea
        aria-describedby={error ? 'message-error' : undefined}
        aria-invalid={Boolean(error)}
        className={`${inputClassName(Boolean(error))} min-h-40 resize-y py-3 leading-6`}
        id="message"
        name="message"
        onBlur={onBlur}
        onChange={(event) => onChange(event.target.value)}
        placeholder="Write the message"
        value={value}
      />
      <FieldError id="message-error" message={error} />
    </label>
  );
}

function FieldError({ id, message }: { id: string; message?: string }) {
  return (
    <span className="mt-2 block min-h-5 text-sm text-rose-700" id={id}>
      {message ?? ''}
    </span>
  );
}

function inputClassName(hasError: boolean) {
  const borderColor = hasError
    ? 'border-rose-500 focus:border-rose-600 focus:ring-rose-200'
    : 'border-stone-300 focus:border-teal-700 focus:ring-teal-100';

  return `block w-full rounded-md border bg-white px-3.5 py-2.5 text-base text-stone-950 shadow-sm outline-none transition placeholder:text-stone-400 focus:ring-4 ${borderColor}`;
}
