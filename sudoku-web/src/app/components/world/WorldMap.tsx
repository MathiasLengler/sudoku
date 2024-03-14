import { useRecoilValue } from "recoil";
import { allWorldCellsState } from "../../state/world";
import Box from "@mui/material/Box";
import { Typography } from "@mui/material";
import { Code } from "../Code";

export const WorldMap = () => {
    const allWorldCells = useRecoilValue(allWorldCellsState);

    // TODO: render world map
    // TODO: change tile

    return (
        <Box className="world-map" sx={{ display: "flex", flexDirection: "column", height: 1 }}>
            <Typography variant="h2">World map</Typography>
            <Code wrap>{JSON.stringify(allWorldCells)}</Code>
        </Box>
    );
};
