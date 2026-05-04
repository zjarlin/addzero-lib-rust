import { createApiClient } from "@addzero/api-client";

import { MSC_AIO_API_BASE_URL } from "./constants";

export const api = createApiClient(MSC_AIO_API_BASE_URL);
