# Privacy Policy

## 1. Introduction

This Privacy Policy explains how the **Network Medicine Extension** (the "Extension") handles user data. The Extension is developed by the **OpenProphetDB Group** (https://www.prophetdb.org/) and is intended solely for **internal use by members of the Group**. We are committed to protecting user privacy and ensuring that no personally identifiable information is collected, shared, or used for commercial purposes.

## 2. Information We Collect

The Extension is designed to support scientific research with a focus on transparency and minimal data collection. Specifically:

- **User-generated Annotations**: Highlights, tags, sentence-level comments, and other annotations made by users on academic literature pages.
- **Contextual Metadata**: Information such as the title of the article, publication source, and structural HTML coordinates of the annotated content.
- **Local Configuration Data**: Interface preferences (e.g., drawer width, visibility state) saved locally using `chrome.storage`.
- **Authentication Metadata**: The Extension uses a third-party authentication provider (Auth0) to verify user identity.

> We do **not** collect or transmit browsing history, personal device data, or login credentials ourselves.

## 3. User Authentication via Auth0

To control access and ensure that only authorized OpenProphetDB members can use the system, the Extension integrates with **Auth0**, a secure third-party identity management platform.

- User authentication (e.g., login email and password) is handled entirely by **Auth0**.  
- OpenProphetDB Group **does not store or manage user credentials** directly.
- Auth0 provides a verified identity token to the Extension, which is used only to:
  - Confirm user identity;
  - Verify group membership;
  - Securely associate user-created annotations with the authenticated user.

All personal data managed by Auth0 is subject to **Auth0’s own privacy and security policies**.

> ⚠️ In the future, user accounts may be provisioned through an **invitation-only system**, to restrict access to verified research collaborators.

## 4. How We Use and Share Information

### 🔹 Data Sync to OpenProphetDB Platform

User-generated annotations and related metadata may be transmitted to **https://drugs.3steps.cn**, the internal research platform of the OpenProphetDB Group. This data is used solely for collaborative academic research among group members.

No personal identity data is transmitted beyond the authentication token provided by Auth0.

### 🔹 No Third-Party Data Sharing

We do **not** share any data with advertisers, analytics providers, or other third parties.  
All data remains within the secure infrastructure of OpenProphetDB.

## 5. Legal Disclosure

We may disclose stored research annotations only if legally required (e.g., a valid subpoena from a government authority). Due to the non-personal and academic nature of the data, this is expected to be extremely rare.

## 6. Data Storage and Security

- Local preferences are stored using Chrome’s `storage` API in the user’s browser.
- Annotation data is transmitted securely via HTTPS to the OpenProphetDB server at **https://drugs.3steps.cn** and stored in a protected research database.
- Authentication is handled by **Auth0**, and only identity tokens are received by the Extension.
- No user passwords or sensitive authentication data are stored by the Extension or by OpenProphetDB Group.

## 7. Internal-Only Use

This Extension is developed for and distributed exclusively within the **OpenProphetDB Group**.  
It is **not intended for public or commercial use**, and any data collected or synchronized is used solely for internal academic collaboration.

## 8. Your Rights

If you are an authenticated user of the OpenProphetDB platform, you may request access to or deletion of your annotations.  
To do so, please contact the project team using the information below.  
You may also delete all locally stored data at any time by removing the Extension or clearing your browser storage.

## 9. Changes to This Policy

We may update this Privacy Policy periodically to reflect changes in platform features or security practices. All updates will be posted at:  
📍 **https://drugs.3steps.cn/assets/README/privacy_policy.html**

Please review this policy periodically to stay informed.

## 10. Contact

For questions or further information about this Privacy Policy, please contact:

**Email:** yjcyxky@gmail.com  
**Website:** https://drugs.3steps.cn
