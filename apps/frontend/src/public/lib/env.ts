import { z } from 'zod';

const schema = z.object({
  VITE_API_BASE_URL: z
    .string()
    .trim()
    .optional()
    .transform((value) => (value ?? '').replace(/\/+$/, '')),
  VITE_SITE_URL: z
    .string()
    .trim()
    .optional()
    .transform((value) => (value ?? 'http://localhost:5174').replace(/\/+$/, '')),
});

const parsed = schema.safeParse(import.meta.env);
if (!parsed.success) {
  console.error('Invalid public portal environment', parsed.error.issues);
  throw new Error('Invalid public portal environment');
}

export const env = parsed.data;

export const apiDocsUrl = `${env.VITE_API_BASE_URL}/swagger-ui`;
