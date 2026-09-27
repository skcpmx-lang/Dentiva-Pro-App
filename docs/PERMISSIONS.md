# Authorization contract and default matrix

Deny by default. Job titles are labels, not runtime authority. Native command guards read grants every request. Owner is a protected role whose grants are explicit rows, not a renderer flag; preserve at least one active owner. Role changes revoke sessions immediately and atomically audit old/new permission sets. A custom role starts with no grants. Per-user deny overrides role allow. User-management does not imply financial access or unrestricted role escalation. Only actors with a grant may delegate it, and only owner may assign ownership/relax security policy.

O=Owner, A=Administrator, D=Dentist, R=Receptionist, C=Accountant, S=Assistant, I=Inventory manager. `x` default grant; `-` no grant. Administrators may be narrowed. Every list/detail/aggregate/report/export/print/search/notification requires the underlying data grants AND action grants. Default reception has NO financial access.

| Permission family (each verb is a separate stored code) | O | A | D | R | C | S | I |
|---|---|---|---|---|---|---|---|
| patients.view | x | x | x | x | x | x | - |
| patients.create, patients.edit | x | x | x | x | - | - | - |
| patients.archive | x | x | - | - | - | - | - |
| patients.delete, data.destroy | x | - | - | - | - | - | - |
| clinical.view, visits.create, visits.edit | x | x | x | - | - | - | - |
| clinical.support | x | x | x | - | - | x | - |
| visits.delete | x | - | - | - | - | - | - |
| chart.view, chart.edit, treatments.record | x | x | x | - | - | - | - |
| prescriptions.view, prescriptions.create, prescriptions.edit | x | x | x | - | - | - | - |
| prescriptions.print | x | x | x | - | - | - | - |
| appointments.view, appointments.create, appointments.edit, queue.manage | x | x | x | x | - | x | - |
| treatments.view | x | x | x | x | x | x | - |
| treatments.manage, treatments.price_override | x | x | - | - | - | - | - |
| invoices.view, invoices.create | x | x | - | - | x | - | - |
| invoices.edit_draft, invoices.discount | x | x | - | - | x | - | - |
| invoices.cancel, invoices.reverse | x | x | - | - | - | - | - |
| payments.view, payments.record | x | x | - | - | x | - | - |
| payments.refund | x | - | - | - | - | - | - |
| accounting.view, accounting.create | x | x | - | - | x | - | - |
| accounting.reverse | x | x | - | - | - | - | - |
| inventory.view, inventory.receive, inventory.issue | x | x | - | - | - | x | x |
| inventory.adjust, suppliers.manage | x | x | - | - | - | - | x |
| staff.view, staff.manage | x | x | - | - | - | - | - |
| staff.salary, staff.identity | x | - | - | - | - | - | - |
| users.manage, roles.manage | x | x | - | - | - | - | - |
| backup.create | x | x | - | - | - | - | - |
| backup.restore | x | - | - | - | - | - | - |
| settings.manage, printers.manage, catalog.manage | x | x | - | - | - | - | - |
| audit.view | x | x | - | - | - | - | - |
| reports.view | x | x | x | - | x | - | x |
| data.export | x | x | x | - | x | - | x |
| documents.print | x | x | x | x | x | - | x |

Attachments inherit both `patients.view` and `clinical.view`, plus attachments.add/open/delete/export as separate grants (O/A/D default add/open, O/A delete, export also requires data.export). Demographic patient DTO excludes medical notes, balances and staff identity. Financial projection requires invoices.view AND payments.view for combined totals; absence means no keys/snippets/counts, not zero values. Clinical.support returns only explicitly delegated support fields, not all clinical notes. Treatment catalog's prices require invoices.view; inventory purchase valuation requires accounting.view. Reports require each contributing dataset grant. Backup is a privileged full-data export, irrespective of other individual data grants, and the UI must warn administrators about that scope.

## Abuse tests
For every command: no session, expired/locked session, inactive user, missing permission, revoked permission during session, malformed input, guessed ID, cross-patient reference, direct invocation, permission field forged in payload. Repeat restricted data probes through patient summary, search, dashboard, notification, report, export and print. Assert denied attempts leave financial and audit mutation state unchanged except a redacted security event. Session renewal cannot unlock a locked session. Frontend timeout is convenience only; native clock is authority.
