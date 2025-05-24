use iced::Renderer;
use iced::widget::Row;

use crate::ui::theme::Element;
use crate::ui::theme::GauntletComplexTheme;
use crate::ui::theme::ThemableWidget;
use crate::ui::theme::get_theme;

pub enum RowStyle {
    ActionShortcut,
    FormInput,
    ListFirstSectionTitle,
    ListSectionTitle,
    GridFirstSectionTitle,
    GridSectionTitle,
    GridItemTitle,
    RootBottomPanel,
    RootTopPanel,
}

impl<'a, Message: 'a> ThemableWidget<'a, Message> for Row<'a, Message, GauntletComplexTheme, Renderer> {
    type Kind = RowStyle;

    fn themed(self, name: RowStyle) -> Element<'a, Message> {
        let theme = get_theme();

        match name {
            RowStyle::ActionShortcut => self.padding(theme.action_shortcut.padding.to_iced()),
            RowStyle::FormInput => self.padding(theme.form_input.padding.to_iced()),
            RowStyle::ListSectionTitle => {
                self.padding(theme.list_section_title.padding.to_iced())
                    .spacing(theme.list_section_title.spacing)
            }
            RowStyle::GridSectionTitle => {
                self.padding(theme.grid_section_title.padding.to_iced())
                    .spacing(theme.grid_section_title.spacing)
            }
            RowStyle::ListFirstSectionTitle => {
                let padding = theme.list_section_title.padding.to_iced();
                self.padding(gauntlet_common_ui::padding(
                    padding.bottom,
                    padding.right,
                    padding.bottom,
                    padding.left,
                ))
                .spacing(theme.list_section_title.spacing)
            }
            RowStyle::GridFirstSectionTitle => {
                let padding = theme.grid_section_title.padding.to_iced();
                self.padding(gauntlet_common_ui::padding(
                    0.0,
                    padding.right,
                    padding.bottom,
                    padding.left,
                ))
                .spacing(theme.grid_section_title.spacing)
            }
            RowStyle::GridItemTitle => self.padding(theme.grid_item_title.padding.to_iced()),
            RowStyle::RootBottomPanel => self.spacing(theme.root_bottom_panel.spacing),
            RowStyle::RootTopPanel => self.spacing(theme.root_top_panel.spacing),
        }
        .into()
    }
}
