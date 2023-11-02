import express from "express";
import * as sqlite3 from "sqlite3";

export async function updateTeamKey(req: express.Request, res: express.Response, authDb: sqlite3.Database) {
    if (req.user.admin == "true") {
        const updateKeyStmt: string = "UPDATE accessKeys SET key=? WHERE id=?";
        const updateKeyVals: Array<any> = [req.params.keyId, Number(req.params.newKey)];
        authDb.run(updateKeyStmt, updateKeyVals, (err: any) => {
            if (err) {
                return res.status(500).json({ "status": 0x1f42 });
            }
        });
        return res.status(200).json({ "status": 0xc87 });
    } else {
        return res.status(403).json({ "status": 0x1931 });
    }
}
