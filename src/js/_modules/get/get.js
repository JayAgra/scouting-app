"use strict";async function _get(e,t){let n=await fetch(e,{method:"GET",cache:"no-cache",credentials:"include",redirect:"follow"});if(403===n.status)throw document.getElementById(t).innerText="access denied",Error("access denied. terminating.");return 401===n.status?window.location.href="/login":204===n.status?document.getElementById(t).innerText="no results":n.ok||(document.getElementById(t).innerText="unhandled error"),n.json()}Object.defineProperty(exports,"__esModule",{value:!0}),exports._get=void 0,exports._get=_get;