import tkinter as tk
from Raspirus.frontend.utility import *  # For colors and fonts


class MainPage(tk.Frame):
    parent_controller = None
    title_label: tk.Label
    drive_selector: tk.Listbox
    start_btn: tk.Button
    info_btn: tk.Button
    settings_btn: tk.Button

    def __init__(self, parent, controller):
        tk.Frame.__init__(self, parent, bg=BACKGROUND_COLOR)

        self.parent_controller = controller

        self.title_label = tk.Label(self, text="RASPIRUS",
                                    font=TITLE_FONT, fg=PRIMARY_COLOR, bg=BACKGROUND_COLOR)
        self.title_label.place(x=140, y=60, width=510, height=120)

        # highlightbackground / highlightcolor sets a border to the component
        self.drive_selector = tk.Listbox(self, highlightbackground=PRIMARY_COLOR, highlightcolor=PRIMARY_COLOR,
                                         highlightthickness=2, bg=TEXT_COLOR)
        self.drive_selector.place(x=90, y=215, width=620, height=48)

        self.start_btn = tk.Button(self, text="START", font=BUTTON_TEXT_FONT,
                                   fg=BACKGROUND_COLOR, bg=PRIMARY_COLOR)
        self.start_btn.config(command=lambda: controller.show_frame(controller.pages[2]))
        self.start_btn.place(x=185, y=315, width=170, height=50)

        self.info_btn = tk.Button(self, text="INFO", font=BUTTON_TEXT_FONT,
                                  fg=BACKGROUND_COLOR, bg=TEXT_COLOR)
        self.info_btn.config(command=lambda: self.open_info_page())
        self.info_btn.place(x=420, y=315, width=170, height=50)

        self.settings_btn = tk.Button(self, text="SETTINGS", font=SMALL_BUTTON_TEXT_FONT,
                                      fg=SECONDARY_COLOR, bg=TEXT_COLOR)
        self.settings_btn.config(command=lambda: controller.show_frame(controller.pages[1]))
        self.settings_btn.place(x=670, y=15, width=110, height=40)

    def open_info_page(self):
        properties = ["NAMEE", None, None, None, "Contact?"]

        info_page = self.parent_controller.pages[3]
        # TODO: Fix this: Function can't be called because the correct frame can't be retrieved
        # info_page.setProperties(info_page, properties)
        self.parent_controller.show_frame(info_page)
