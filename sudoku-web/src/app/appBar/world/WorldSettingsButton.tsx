import LanguageIcon from "@mui/icons-material/Language";
import assertNever from "assert-never";
import { useRecoilState } from "recoil";
import MyIconButton from "../../components/MyIconButton";
import { MyMenu } from "../../components/MyMenu";
import { gameState } from "../../state/gameMode";
import { DEFAULT_WORLD_GRID_POSITION } from "../../state/world";
import { usePlaySelectedGrid, useShowWorldMap } from "../../actions/worldActions";

export function WorldSettingsButton() {
    const [game, setGame] = useRecoilState(gameState);

    const playSelectedGrid = usePlaySelectedGrid();
    const showWorldMap = useShowWorldMap();

    return (
        <MyMenu
            menuItems={[
                {
                    label: `Toggle game mode (${game.mode})`,
                    // icon: <OpenInNewIcon />,
                    onClick: () => {
                        setGame((gameMode) => {
                            if (gameMode.mode === "sudoku") {
                                return {
                                    mode: "world",
                                    view: "sudoku",
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
