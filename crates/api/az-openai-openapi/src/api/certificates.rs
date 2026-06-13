//! Certificates REST endpoint contract.

use async_trait::async_trait;

use crate::bodies::*;

/// Certificates REST endpoints.
#[async_trait]
pub trait OpenAiCertificatesApi: Send + Sync {
    /// Error type returned by the application-layer implementation.
    type Error: std::error::Error + Send + Sync + 'static;
    /// List uploaded certificates for this organization.
    ///
    /// REST: `GET /organization/certificates`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_CERTIFICATES`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_CERTIFICATES).
    async fn list_organization_certificates(
        &self,
        limit: Option<i64>,
        after: Option<String>,
        order: Option<String>,
    ) -> Result<OpenAiResponseBody, Self::Error>;

    /// Upload a certificate to the organization. This does **not** automatically activate the certificate. Organizations can upload up to 50 certificates.
    ///
    /// REST: `POST /organization/certificates`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_CERTIFICATES`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_CERTIFICATES).
    async fn upload_certificate(
        &self,
        body: OpenAiRequestBody,
    ) -> Result<OpenAiResponseBody, Self::Error>;

    /// Activate certificates at the organization level. You can atomically and idempotently activate up to 10 certificates at a time.
    ///
    /// REST: `POST /organization/certificates/activate`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_CERTIFICATES_BY_ACTIVATE`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_CERTIFICATES_BY_ACTIVATE).
    async fn activate_organization_certificates(
        &self,
        body: OpenAiRequestBody,
    ) -> Result<OpenAiResponseBody, Self::Error>;

    /// Deactivate certificates at the organization level. You can atomically and idempotently deactivate up to 10 certificates at a time.
    ///
    /// REST: `POST /organization/certificates/deactivate`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_CERTIFICATES_BY_DEACTIVATE`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_CERTIFICATES_BY_DEACTIVATE).
    async fn deactivate_organization_certificates(
        &self,
        body: OpenAiRequestBody,
    ) -> Result<OpenAiResponseBody, Self::Error>;

    /// Delete a certificate from the organization. The certificate must be inactive for the organization and all projects.
    ///
    /// REST: `DELETE /organization/certificates/{certificate_id}`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_CERTIFICATES_BY_CERTIFICATE_ID`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_CERTIFICATES_BY_CERTIFICATE_ID).
    async fn delete_certificate(
        &self,
        certificate_id: String,
    ) -> Result<OpenAiResponseBody, Self::Error>;

    /// Get a certificate that has been uploaded to the organization. You can get a certificate regardless of whether it is active or not.
    ///
    /// REST: `GET /organization/certificates/{certificate_id}`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_CERTIFICATES_BY_CERTIFICATE_ID`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_CERTIFICATES_BY_CERTIFICATE_ID).
    async fn get_certificate(
        &self,
        certificate_id: String,
        include: Option<Vec<String>>,
    ) -> Result<OpenAiResponseBody, Self::Error>;

    /// Modify a certificate. Note that only the name can be modified.
    ///
    /// REST: `POST /organization/certificates/{certificate_id}`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_CERTIFICATES_BY_CERTIFICATE_ID`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_CERTIFICATES_BY_CERTIFICATE_ID).
    async fn modify_certificate(
        &self,
        certificate_id: String,
        body: OpenAiRequestBody,
    ) -> Result<OpenAiResponseBody, Self::Error>;

    /// List certificates for this project.
    ///
    /// REST: `GET /organization/projects/{project_id}/certificates`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_CERTIFICATES`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_CERTIFICATES).
    async fn list_project_certificates(
        &self,
        project_id: String,
        limit: Option<i64>,
        after: Option<String>,
        order: Option<String>,
    ) -> Result<OpenAiResponseBody, Self::Error>;

    /// Activate certificates at the project level. You can atomically and idempotently activate up to 10 certificates at a time.
    ///
    /// REST: `POST /organization/projects/{project_id}/certificates/activate`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_CERTIFICATES_BY_ACTIVATE`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_CERTIFICATES_BY_ACTIVATE).
    async fn activate_project_certificates(
        &self,
        project_id: String,
        body: OpenAiRequestBody,
    ) -> Result<OpenAiResponseBody, Self::Error>;

    /// Deactivate certificates at the project level. You can atomically and idempotently deactivate up to 10 certificates at a time.
    ///
    /// REST: `POST /organization/projects/{project_id}/certificates/deactivate`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_CERTIFICATES_BY_DEACTIVATE`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_CERTIFICATES_BY_DEACTIVATE).
    async fn deactivate_project_certificates(
        &self,
        project_id: String,
        body: OpenAiRequestBody,
    ) -> Result<OpenAiResponseBody, Self::Error>;
}
