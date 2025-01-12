import LanguageIcon from "@mui/icons-material/Language";
import assertNever from "assert-never";
import { useRecoilState } from "recoil";
import MyIconButton from "../../components/MyIconButton";
import { MyMenu } from "../../components/MyMenu";
import { gameState } from "../../state/gameMode";
import { DEFAULT_WORLD_GRID_POSITION, requestedGridDimState, worldGridDimSchema } from "../../state/world";
import { usePlaySelectedGrid, useShowWorldMap } from "../../actions/worldActions";
import _ from "lodash";
import { z } from "zod";

const gridDims = z.array(worldGridDimSchema).parse([
    { rowCount: 3, columnCount: 3 },
    { rowCount: 10, columnCount: 10 },
    { rowCount: 20, columnCount: 20 },
    { rowCount: 50, columnCount: 50 },
]);

export function WorldSettingsButton() {
    const [game, setGame] = useRecoilState(gameState);
    const [requestedGridDim, setRequestedGridDim] = useRecoilState(requestedGridDimState);

    const playSelectedGrid = usePlaySelectedGrid();
    const showWorldMap = useShowWorldMap();

    return (
        <MyMenu
            menuItems={[
                {
                    label: `Toggle game mode (${game.mode})`,
                    onClick: () => {
                        setGame((gameMode) => {
                            if (gameMode.mode === "sudoku") {
                                return {
                                    mode: "world",
                                    view: "map",
                                    selectedGridPosition: DEFAULT_WORLD_GRID_POSITION,
                                };
                            } else if (gameMode.mode === "world") {
                                return {
                                    mode: "sudoku",
                                };
                            } else {
                                assertNever(gameMode);
                            }
                        });
                    },
                },
                ...(game.mode === "world"
                    ? [
                          {
                              label: `Toggle world view (${game.view})`,
                              onClick: async () => {
                                  if (game.view === "sudoku") {
                                      await showWorldMap();
                                  } else if (game.view === "map") {
                                      await playSelectedGrid();
                                  } else {
                                      assertNever(game.view);
                                  }
                              },
                          },
                          {
                              label: `Toggle world size (${requestedGridDim.rowCount}x${requestedGridDim.columnCount})`,
                              onClick: () => {
                                  setRequestedGridDim((gridDim) => {
                                      const currentIndex = _.findIndex(gridDims, gridDim);

                                      if (currentIndex === -1) {
                                          return _.head(gridDims)!;
                                      } else {
                                          return gridDims[(currentIndex + 1) % gridDims.length]!;
                                      }
                                  });
                              },
                          },
                      ]
                    : []),
            ]}
        >
            {({ onMenuOpen }) => (
                <MyIconButton
                    label="World settings"
                    icon={LanguageIcon}
                    size="large"
                    color="inherit"
                    onClick={onMenuOpen}
                />
            )}
        </MyMenu>
    );
}
