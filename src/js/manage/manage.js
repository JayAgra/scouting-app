import { _get } from "../_modules/get/get.min.js";
export function newSearch() {
    location.reload();
}
export async function getSubmissionData() {
    document.getElementById("viewData").innerHTML = "requesting...";
    _get("/api/manage/list", "viewData").then((response) => {
        var listHTML = "";
        for (var i = response.length - 1; i >= 0; i--) {
            listHTML = listHTML + `<fieldset><span><span>ID:&emsp;${response[i].id}</span>&emsp;&emsp;<span><a href="/detail?id=${response[i].id}" style="all: unset; color: #2997FF; text-decoration: none;">View</a>&emsp;<span onclick="deleteSubmission(${response[i].id}, 'main${response[i].id}')" style="color: red" id="main${response[i].id}">Delete</span></span></span></fieldset>`;
        }
        document.getElementById("resultsInsert").insertAdjacentHTML("afterbegin", listHTML);
        document.getElementById("search").style.display = "none";
        document.getElementById("results").style.display = "flex";
    });
}
export async function deleteSubmission(submission, linkID) {
    document.getElementById(linkID).innerHTML = "deleting...";
    _get(`/api/manage/${submission}/delete}`, linkID).then((response) => {
        if (response.status === 0xc83) {
            document.getElementById(linkID).innerHTML = "deleted!";
        }
    });
}
window.newSearch = newSearch;
window.getSubmissionData = getSubmissionData;
window.deleteSubmission = deleteSubmission;
