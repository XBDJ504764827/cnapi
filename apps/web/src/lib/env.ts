import { z } from 'zod';

/**
 * Environment variables are validated once at startup. Invalid configuration
 * fails loudly instead of silently falling back to a wrong API host.
 */
const envSchema = z.object({
  VITE_APP_NAME: z.string().trim().min(1).default('CNGOKZAPI'),
  /** Empty string = same origin (dev proxy / production static server proxy). */
  VITE_API_BASE_URL: z
    .string()
    .trim()
    .optional()
    .transform((value) => (value ?? '').replace(/\/+$/, '')),
  VITE_API_DOCS_PATH: z.string().trim().min(1).default('/swagger-ui'),
});

const parsed = envSchema.safeParse(import.meta.env);

if (!parsed.success) {
  console.error('Invalid environment configuration', parsed.error.issues);
  throw new Error('Invalid environment configuration for the web application');
}

export const env = parsed.data;

/** Absolute or same-origin URL of the Swagger UI. */
export const apiDocsUrl = `${env.VITE_API_BASE_URL}${env.VITE_API_DOCS_PATH}`;

/** Absolute or same-origin URL of the generated OpenAPI document. */
export const openApiDocumentUrl = `${env.VITE_API_BASE_URL}/api-docs/openapi.json`;
