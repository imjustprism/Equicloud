# Privacy Policy

**Effective Date:** September 24, 2025
**Last Updated:** September 24, 2025

## Overview

EquiCloud is a settings backup service that allows users to store and sync their application settings. This privacy policy explains how we collect, use, and protect your data.

## Data We Collect

### Authentication Data
- **Discord User ID**: Collected via Discord OAuth to identify and authenticate users
- **Discord Access Token**: Temporarily used during authentication (not stored)

### User Settings Data
- **Settings Files**: Application settings and configurations you choose to backup
- **Metadata**: File creation and modification timestamps
- **File Size Information**: To enforce storage limits

## How We Use Your Data

- **Authentication**: Verify your identity and provide secure access to your settings
- **Settings Storage**: Store and retrieve your backed-up settings

## Data Storage, Security, and Retention

- **Database**: User data is stored in ScyllaDB with CRC32 hashed user identifiers
- **Access Control**: Only authenticated users can access their own data; administrators may restrict access to whitelisted users
- **File Size Limits**: Settings are limited to 60MB by default to prevent abuse
- **Data Retention**: Settings stored until you delete them; Discord tokens are not permanently stored

## Your Rights

You have the right to:
- **Access**: View your stored settings data
- **Delete**: Remove your settings using `DELETE /v1/settings` or all data using `DELETE /v1`
- **Modify**: Update your settings at any time

## Third-Party Services

- **Discord**: Used for user authentication via OAuth2
- **ScyllaDB**: Database service for data storage

## Data Sharing

We do not sell, trade, or share your personal data with third parties except:
- When required by law
- To protect our rights or safety

## Administrator Controls

Service administrators may:
- Configure user access restrictions via Discord user ID whitelist
- Set file size limits
- Access system logs for maintenance

## Contact Information

For privacy-related questions or to exercise your rights, please contact the service administrator.

## Changes to This Policy

We may update this privacy policy occasionally. Users will be notified of significant changes through the service.

---

*This privacy policy is effective as of the date listed above and applies to all users of the EquiCloud service.*