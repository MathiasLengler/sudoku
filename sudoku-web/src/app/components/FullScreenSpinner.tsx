import { Stack } from "@mui/material";

import CircularProgress from "@mui/material/CircularProgress";

export function FullScreenSpinner() {
    return (
        <div className="app-spinner" sx={{ height: 1 }}>
            <Stack
                direction="column"
                spacing={2}
                sx={{
                    justifyContent: "center",
                    alignItems: "center",
                    height: 1,
                }}
            >
                <CircularProgress />
            </Stack>
        </div>
    );
}
