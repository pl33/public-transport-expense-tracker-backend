from typing import *

from pydantic import BaseModel, Field


class TagOption(BaseModel):
    """
    None model
        JSON structure

    """

    model_config = {"populate_by_name": True, "validate_assignment": True}

    id: Optional[int] = Field(validation_alias="id", default=None)

    tag_id: Optional[int] = Field(validation_alias="tag_id", default=None)

    order: int = Field(validation_alias="order")

    value: str = Field(validation_alias="value")

    uuid: Optional[str] = Field(validation_alias="uuid", default=None)

    name: Optional[str] = Field(validation_alias="name", default=None)

    display_name: Optional[str] = Field(validation_alias="display_name", default=None)
