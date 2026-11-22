/**
 * The look of a chat exported as HTML. An exported file stands on its own and
 * is printed or opened anywhere, so it carries its own plain stylesheet
 * instead of the dashboard's theme. It lives here so every visual decision the
 * dashboard makes is in this folder.
 */
export const exportDocumentStyle =
  "body{max-width:850px;margin:40px auto;padding:0 24px;font:16px/1.6 system-ui;color:#222}header{border-bottom:1px solid #ccc}article{margin:32px 0}h2{font-size:14px;color:#555}pre{white-space:pre-wrap;overflow-wrap:anywhere;background:#f5f5f5;padding:16px}table{border-collapse:collapse}td,th{border:1px solid #ccc;padding:6px}img{display:none}@media print{body{margin:0;max-width:none}pre{white-space:pre-wrap}a{color:inherit}}";
