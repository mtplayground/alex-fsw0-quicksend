import { expect, test } from '@playwright/test';

type SendRequestBody = {
  recipient_email?: string;
  subject?: string;
  message?: string;
};

test('submits the form and renders the success message', async ({ page }) => {
  await page.route('**/api/send', async (route) => {
    expect(route.request().method()).toBe('POST');
    const body = route.request().postDataJSON() as SendRequestBody;

    expect(body).toEqual({
      recipient_email: 'person@example.com',
      subject: 'Project update',
      message: 'The launch checklist is ready.',
    });

    await route.fulfill({
      contentType: 'application/json',
      status: 202,
      body: JSON.stringify({
        status: 'accepted',
        message: 'Message delivered.',
        delivery_status: 'sent',
        message_id: 'msg_e2e_success',
      }),
    });
  });

  await page.goto('/');
  await page.getByLabel('Recipient email').fill('person@example.com');
  await page.getByLabel('Subject').fill('Project update');
  await page.getByLabel('Message').fill('The launch checklist is ready.');
  await page.getByRole('button', { name: 'Send' }).click();

  await expect(page.getByRole('status')).toContainText('Message delivered.');
  await expect(page.getByLabel('Recipient email')).toHaveValue('');
  await expect(page.getByLabel('Subject')).toHaveValue('');
  await expect(page.getByLabel('Message')).toHaveValue('');
});

test('shows validation errors without calling the API', async ({ page }) => {
  let apiCalled = false;

  await page.route('**/api/send', async (route) => {
    apiCalled = true;
    await route.abort();
  });

  await page.goto('/');
  await page.getByRole('button', { name: 'Send' }).click();

  await expect(page.getByText('Review the highlighted fields.')).toBeVisible();
  await expect(page.getByText('Enter a valid email address.')).toBeVisible();
  await expect(page.getByText('Enter a subject.')).toBeVisible();
  await expect(page.getByText('Enter a message.')).toBeVisible();
  expect(apiCalled).toBe(false);
});

test('shows a try-again-later message when rate limited', async ({ page }) => {
  await page.route('**/api/send', async (route) => {
    await route.fulfill({
      contentType: 'application/json',
      status: 429,
      body: JSON.stringify({
        error: {
          code: 'rate_limited',
          message: 'Too many send attempts. Try again later.',
        },
      }),
    });
  });

  await page.goto('/');
  await page.getByLabel('Recipient email').fill('person@example.com');
  await page.getByLabel('Subject').fill('Project update');
  await page.getByLabel('Message').fill('The launch checklist is ready.');
  await page.getByRole('button', { name: 'Send' }).click();

  await expect(page.getByRole('alert')).toContainText(
    'Too many attempts. Try again later.',
  );
});
