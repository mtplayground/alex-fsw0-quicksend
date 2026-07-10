export type SendFormValues = {
  recipientEmail: string;
  subject: string;
  message: string;
};

export type SendFormErrors = Partial<Record<keyof SendFormValues, string>>;

export function validateSendForm(values: SendFormValues): SendFormErrors {
  const errors: SendFormErrors = {};

  if (!isValidEmail(values.recipientEmail)) {
    errors.recipientEmail = 'Enter a valid email address.';
  }

  if (values.subject.trim().length === 0) {
    errors.subject = 'Enter a subject.';
  }

  if (values.message.trim().length === 0) {
    errors.message = 'Enter a message.';
  }

  return errors;
}

export function isValidEmail(value: string) {
  const email = value.trim();

  if (email.length === 0 || email !== value || /\s/.test(email)) {
    return false;
  }

  return /^[A-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[A-Z0-9-]+(?:\.[A-Z0-9-]+)+$/i.test(
    email,
  );
}
